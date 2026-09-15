//! slug.rs — Slug-Hash 生成算法。
//!
//! 算法规范见 `docs/tech/ARCHITECTURE.md` 第 4 节，本文件是它在 Rust 侧的落地。
//!
//! # 为什么需要它
//!
//! 知识库文件名是中文的（`Vue3 组件性能优化.md`、`C++ #include 踩坑记录.md`），
//! 直接拿文件名当目录名会有三类问题：
//! 1. 中文、空格、`#`、`+` 在部分工具链上解析不稳定，甚至被当成路径分隔或注释符
//! 2. 文件名改了无法追踪——旧目录残留、新目录重写，用户看到两份
//! 3. 不同文件可能算出同一个目录名（重名覆盖）
//!
//! 解法是把「文件名」与「内容」分别折算进同一个标识符：
//!
//! ```text
//! crossbrain-{文件名折算的 kebab 段}-{内容 SHA-256 的前 6 位}
//!    │                │                        │
//!    │                │                        └─ 内容变了 → 换目录 → 旧目录成为孤儿被清理
//!    │                └─ 纯 ASCII，仅 [a-z0-9-]，≤30 字符
//!    └─ 命名空间前缀，与 adapters::ensure_crossbrain_slug 的边界同源
//! ```
//!
//! # 不变性保证（ARCHITECTURE 第 4 节）
//!
//! | 变更 | kebab 段 | hash 段 | 结果 |
//! |:---|:---|:---|:---|
//! | 只改文件名 | 变 | 不变 | 新目录；旧目录被孤儿清理删除 |
//! | 只改内容 | 不变 | 变 | 新目录；旧目录被孤儿清理删除 |
//! | 都不改 | 不变 | 不变 | **同一目录，幂等**（不产生新目录、不产生孤儿） |
//!
//! # 已知取舍
//!
//! - **不做拼音转换**：`ARCHITECTURE` 第 4 节的示例把「组件性能优化」折算成了
//!   `perf`，那是人工示意而非算法产物。V1 不引入拼音词典（体积与维护成本都不划算），
//!   中文一律被折叠掉，因此 `Vue3 组件性能优化.md` 实际得到 kebab 段 `vue3`。
//!   文件名里若全是中文，kebab 段会退化为 [`FALLBACK_KEBAB`]——**功能不受影响**，
//!   区分度由 hash 段承担。
//! - **hash 段只有 24 bit**（6 位十六进制）：碰撞概率约 1/1677 万。
//!   碰撞的后果是「两个知识共用一个技能目录」，属可接受的退化；
//!   而目录名更短既利于阅读，也远离 Windows 路径长度上限。

use sha2::{Digest, Sha256};

/// CrossBrain 托管内容的统一命名空间前缀。
///
/// 定义在**这里**而不是 `adapters/mod.rs`，因为它是 slug 的形态本身——
/// Adapter 层的命名空间校验（[`crate::adapters::ensure_crossbrain_slug`]）
/// 是**引用**它，而不是另立一份。两处若各写一个字符串常量，
/// 日后改动一侧就会出现「生成的名字校验不过」或「校验放过了不该放过的名字」。
pub const SLUG_PREFIX: &str = "crossbrain-";

/// kebab 段的最大**字符数**（ARCHITECTURE 第 4 节：截断至 ≤30 字符）。
///
/// 取值理由：目录名总长为 `11 + 30 + 1 + 6 = 48`，
/// 满足 `TEST_PLAN.md` T1-1「总长 ≤ 50 字符」，同时远离 Windows MAX_PATH 风险区。
pub const MAX_KEBAB_LEN: usize = 30;

/// hash 段长度（SHA-256 前 3 字节 = 6 位十六进制）。
pub const HASH_LEN: usize = 6;

/// 文件名折算不出任何 ASCII 片段时的兜底 kebab 段。
///
/// 触发场景：`优化.md`、`!!!.md`、`···.md` 这类**纯中文/纯符号**文件名。
/// 不兜底的话 kebab 段为空，目录名会变成 `crossbrain--7c8e2a`（双横线），
/// 虽然合法但形态难看，且让「分隔符」失去区分作用。
pub const FALLBACK_KEBAB: &str = "item";

/// 一个完整的 slug-hash 及其各组成部分。
///
/// # 为什么要按段暴露，而不是只给一个字符串
///
/// 同一个 slug-hash 在两个地方被使用，且**需要的形态不同**：
///
/// | 用途 | 需要的形态 | 取值 |
/// |:---|:---|:---|
/// | `Adapter::sync_l2()` / `cleanup_orphans()` 的目录名 | 含命名空间前缀 | [`Slug::dir_name`] |
/// | `format_skill_md()` 写入 frontmatter 的 `name` 字段 | **不含**前缀 | [`Slug::skill_name`] |
///
/// 若只交付一个字符串，上层就得靠 `strip_suffix` / `split('-')` 反推各段——
/// 而 kebab 段自身就含 `-`，切割位置无从判断。分段交付把这件事在源头固定下来。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slug {
    /// 纯 kebab 段，如 `"vue3"`（不含 `crossbrain-`，不含 hash）
    kebab: String,
    /// hash 段，如 `"7c8e2a"`（6 位十六进制，不含分隔符）
    hash: String,
    /// 完整目录名，如 `"crossbrain-vue3-7c8e2a"`
    dir_name: String,
}

impl Slug {
    /// 完整标识符：`crossbrain-{kebab}-{hash}`。
    ///
    /// 用于 `sync_l2` 的目标目录名、`cleanup_orphans` 的 `active_slugs` 列表。
    /// 该值**保证**能通过 [`crate::adapters::ensure_crossbrain_slug`] 校验
    /// （由本模块的契约测试锁定）。
    pub fn dir_name(&self) -> &str {
        &self.dir_name
    }

    /// 写入 SKILL.md frontmatter `name` 字段的技能名：`{kebab}`。
    ///
    /// **不含** `crossbrain-` 前缀。依据 `ADAPTER_SPEC.md` 第 4 节的字段表与示例
    /// （`name: vue3-perf`）——该字段是给 AI 看的技能名，不是文件系统路径，
    /// 带上工具专有的命名空间前缀只会污染语义。
    pub fn skill_name(&self) -> &str {
        &self.kebab
    }

    /// 纯 kebab 段（等价于 [`Slug::skill_name`]，语义更直白时使用）。
    pub fn kebab(&self) -> &str {
        &self.kebab
    }

    /// hash 段（6 位十六进制）。
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// 与 [`Slug::dir_name`] 等价，便于直接当 `&str` 使用。
    pub fn as_str(&self) -> &str {
        &self.dir_name
    }
}

/// 展示形态即完整目录名——它是 slug-hash 的规范写法，
/// 日志、错误信息与 UI 都应展示同一个值，不做二次加工。
impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.dir_name)
    }
}

/// 生成 slug-hash 的完整结构（推荐入口）。
///
/// 需要同时拿到「目录名」与「frontmatter 技能名」时用这个，
/// 避免上层对完整标识符做字符串切割。
pub fn generate_parts(filename: &str, content: &str) -> Slug {
    let kebab = filename_to_kebab(filename);
    let hash = content_hash(content);
    let dir_name = format!("{SLUG_PREFIX}{kebab}-{hash}");
    Slug {
        kebab,
        hash,
        dir_name,
    }
}

/// 将用户输入的标题折算为知识库文件名的 kebab 段（TASK-12）。
///
/// 这是 [`filename_to_kebab`] 的公开入口：新建技能知识时，文件名 =
/// `{kebab}.md`（不含 hash 段——同一段 kebab 会随内容变化换目录，
/// 文件名层再带 hash 只会制造「改一个字文件就改名」的错觉；
/// 不含 `crossbrain-` 前缀——文件名折算时前缀会被原样保留，
/// 叠加成 `crossbrain-crossbrain-…`）。复用同一套折算规则保证
/// 「文件名」与「注入目录名的 kebab 段」形态一致。
///
/// 纯中文标题会退化为 [`FALLBACK_KEBAB`]，调用方必须自行去重
/// （`sync.rs` 的 `create_knowledge_for` 按 `-2`、`-3` 递增）。
pub fn kebab_from_title(title: &str) -> String {
    filename_to_kebab(title)
}

/// 从知识文件名和内容生成 `crossbrain-{kebab}-{hash}` 标识符。
///
/// # 参数
/// - `filename`：知识文件名，如 `"Vue3 组件性能优化.md"`（可含中文、空格、扩展名）
/// - `content`：文件完整内容（**逐字节敏感**：改一个空格也会换 hash）
///
/// # 返回
/// 形如 `"crossbrain-vue3-7c8e2a"` 的字符串。等价于
/// `generate_parts(filename, content).dir_name().to_string()`。
///
/// # 幂等
/// 同输入必得同输出，无时间戳、无随机数、无环境依赖。
pub fn generate(filename: &str, content: &str) -> String {
    generate_parts(filename, content).dir_name
}

/// 将文件名折算为安全的 kebab-case 段。
///
/// 步骤（顺序不可颠倒）：
/// 1. 剥离 `.md` 扩展名（大小写不敏感，如 `.MD`、`.Md`）
/// 2. ASCII 字母数字保留并转小写；**其余所有字符**（中文、空格、`#`、`+`、`_`、`.`…）
///    折叠为 `-`
/// 3. 合并连续 `-`，去掉首尾 `-`
/// 4. 截断至 [`MAX_KEBAB_LEN`] 个字符，再去掉可能被截出来的尾部 `-`
/// 5. 若结果为空，回退为 [`FALLBACK_KEBAB`]
///
/// # 为什么逐字符处理而不是正则
///
/// 中文是多字节字符。任何以**字节**为单位的下标运算
/// （如 `name[name.len() - 3..]`）都可能切在字符中间并 panic——
/// 这里所有切割都走 [`str::is_char_boundary`] 守卫或 `char` 迭代器。
fn filename_to_kebab(filename: &str) -> String {
    let name = strip_md_extension(filename);

    let mut kebab = String::with_capacity(name.len());
    let mut prev_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            kebab.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !kebab.is_empty() && !prev_dash {
            // 前导 `-` 不写入（`kebab.is_empty()` 拦截），连续 `-` 只写一次
            kebab.push('-');
            prev_dash = true;
        }
        // 其余情况（前导分隔符、连续分隔符）直接跳过
    }
    while kebab.ends_with('-') {
        kebab.pop();
    }

    // 截断：按字符数取前 MAX_KEBAB_LEN 个。此时 kebab 只含 ASCII，
    // 字符数与字节数相等——但仍用 chars() 表达意图，避免日后放宽字符集时踩坑。
    if kebab.chars().count() > MAX_KEBAB_LEN {
        kebab = kebab.chars().take(MAX_KEBAB_LEN).collect();
        while kebab.ends_with('-') {
            kebab.pop();
        }
    }

    if kebab.is_empty() {
        return FALLBACK_KEBAB.to_string();
    }
    kebab
}

/// 剥离 `.md` 扩展名（大小写不敏感）。
///
/// 只认 `.md`，**不认** `.markdown`：知识库扫描规则是 `knowledge/*.md`，
/// 若这里接受 `.markdown`，就会为一批「永远不会被扫描到」的文件生成 slug，
/// 属于制造死路径。
///
/// 长度不足 3 字节或以中文结尾时，`len - 3` 可能落在字符中间——
/// 必须先用 [`str::is_char_boundary`] 守卫，否则 `&filename[..idx]` 会 panic。
fn strip_md_extension(filename: &str) -> &str {
    let Some(idx) = filename.len().checked_sub(3) else {
        return filename; // 短于 ".md"，不可能带该扩展名
    };
    if filename.is_char_boundary(idx) && filename[idx..].eq_ignore_ascii_case(".md") {
        &filename[..idx]
    } else {
        filename
    }
}

/// 计算内容指纹：SHA-256 的前 3 字节，表示为 6 位小写十六进制。
///
/// 用 SHA-256 而非 `DefaultHasher` 这类进程内哈希：后者的输出在不同 Rust 版本
/// 之间**不保证稳定**，会导致升级工具链后全量技能目录被判定为孤儿而删除重建。
/// 内容寻址的标识符必须是可跨版本、跨机器复现的——这是 SHA 的用途所在。
fn content_hash(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    // digest 长度固定 32 字节，前 3 字节必然存在
    format!("{:02x}{:02x}{:02x}", digest[0], digest[1], digest[2])
}

#[cfg(test)]
mod tests {
    //! 用例编号对应 `docs/testing/TEST_PLAN.md` 的 T1 组（T1-1 ~ T1-4）。

    use super::*;

    // ---------- T1-1 基础生成 ----------

    /// T1-1：格式正确，且总长满足文档约束（≤50 字符）。
    #[test]
    fn t1_1_generates_well_formed_slug() {
        let slug = generate_parts("Vue3 组件性能优化.md", "test");

        assert!(
            slug.dir_name().starts_with(SLUG_PREFIX),
            "必须以命名空间前缀开头：{}",
            slug.dir_name()
        );
        assert_eq!(slug.kebab(), "vue3", "中文部分会被折叠掉（V1 不做拼音）");
        assert_eq!(slug.hash().len(), HASH_LEN);
        assert_eq!(slug.dir_name(), format!("crossbrain-vue3-{}", slug.hash()));

        // 最长形态：crossbrain-(11) + kebab(30) + -(1) + hash(6) = 48
        let longest = generate_parts(&format!("{}.md", "a".repeat(200)), "x");
        assert!(
            longest.dir_name().chars().count() <= 50,
            "总长超出文档约束：{}",
            longest.dir_name().chars().count()
        );
        assert_eq!(longest.kebab().chars().count(), MAX_KEBAB_LEN);
    }

    /// T1-1 的 hash 必须是真的 SHA-256 前 3 字节，而不是任何占位实现。
    /// `"abc"` 与空串是标准测试向量，值可外部复现。
    #[test]
    fn t1_1_hash_matches_sha256_test_vectors() {
        assert_eq!(content_hash("abc"), "ba7816");
        assert_eq!(content_hash(""), "e3b0c4");
    }

    // ---------- T1-2 幂等性 ----------

    /// T1-2：同输入三次，输出完全相同。
    #[test]
    fn t1_2_is_idempotent() {
        let a = generate("Vue3 组件性能优化.md", "内容A");
        let b = generate("Vue3 组件性能优化.md", "内容A");
        let c = generate("Vue3 组件性能优化.md", "内容A");
        assert_eq!(a, b);
        assert_eq!(b, c);

        // 结构体整段相等（含各分段），幂等不能只体现在拼接结果上
        assert_eq!(
            generate_parts("a.md", "x"),
            generate_parts("a.md", "x"),
            "Slug 各分段也应逐字段相等"
        );
    }

    // ---------- T1-3 内容变更触发 hash 变更 ----------

    /// T1-3：同一文件名，内容变一行 → 后 6 位不同，kebab 段不变。
    #[test]
    fn t1_3_content_change_changes_hash_only() {
        let before = generate_parts("note.md", "# 标题\n正文");
        let after = generate_parts("note.md", "# 标题\n正文\n新增一行");

        assert_ne!(
            before.hash(),
            after.hash(),
            "内容变更必须换 hash（旧目录才会成为孤儿被清理）"
        );
        assert_eq!(
            before.kebab(),
            after.kebab(),
            "只改内容时 kebab 段不应变化"
        );
    }

    /// 文件名变更 → kebab 段变、hash 段不变（内容未动）。
    #[test]
    fn t1_3_rename_changes_kebab_only() {
        let before = generate_parts("alpha.md", "同样内容");
        let after = generate_parts("beta.md", "同样内容");

        assert_ne!(before.kebab(), after.kebab(), "改名应换 kebab 段");
        assert_eq!(before.hash(), after.hash(), "内容未动则 hash 不变");
    }

    // ---------- T1-4 中文 / 特殊字符安全 ----------

    /// T1-4：输出全为 ASCII，且只含 `a-z0-9-`。
    #[test]
    fn t1_4_output_is_path_safe_ascii() {
        let samples = [
            "Rust · 生命周期与借用检查器.md",
            "C++ #include 踩坑记录.md",
            "Vue3 组件性能优化.md",
            " 空格开头和结尾 .md",
            "a/b\\c:d*e?f\"g<h>i|j.md",
            "emoji 🚀 也要安全.md",
        ];

        for filename in samples {
            let slug = generate(filename, "内容");
            assert!(
                slug.is_ascii(),
                "「{filename}」产出非 ASCII：{slug}"
            );
            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "「{filename}」产出非法字符：{slug}"
            );
            assert!(
                !slug.contains("--"),
                "「{filename}」出现连续横线（说明折叠逻辑失效）：{slug}"
            );
            assert!(
                !slug.ends_with('-'),
                "「{filename}」以横线结尾：{slug}"
            );
        }
    }

    /// 文件名里的路径分隔符绝不能被带进标识符——否则目录名会跨层。
    #[test]
    fn t1_4_slash_is_neutralised() {
        let slug = generate("../../etc/passwd.md", "x");
        assert!(!slug.contains('/'), "斜杠未被折叠：{slug}");
        assert!(!slug.contains(".."), "上跳序列未被消除：{slug}");
    }

    // ---------- 边界与退化（文档未覆盖，实测补入）----------

    /// 纯中文文件名：kebab 段会退化为兜底值，但**不 panic**、格式仍合法。
    ///
    /// 这是本机最可能的真实输入——用户的 knowledge/ 里全是中文文件名。
    #[test]
    fn pure_chinese_filename_falls_back_without_panic() {
        for filename in ["优化.md", "组件性能优化.md", "！！！.md", "···.md", ".md", ""] {
            let slug = generate_parts(filename, "内容");
            assert_eq!(
                slug.kebab(),
                FALLBACK_KEBAB,
                "「{filename}」应回退到 {FALLBACK_KEBAB}：{slug}"
            );
            assert_eq!(slug.dir_name(), format!("crossbrain-{FALLBACK_KEBAB}-{}", slug.hash()));
        }
    }

    /// `strip_md_extension` 的字节下标必须落在字符边界上。
    ///
    /// 反例：`"优化"` 是 6 字节，`len - 3 = 3` 落在第二个汉字中间——
    /// 若直接用 `&filename[..3]` 会 panic。这里用 2 字符中文名专门踩这个点。
    #[test]
    fn chinese_filename_does_not_panic_on_byte_slicing() {
        // 若实现存在字节切片缺陷，这几行会 panic 而不是断言失败
        let _ = generate("优化.md", "x");
        let _ = generate("优化", "x");
        let _ = generate("中文", "x");
        let _ = generate("🎉.md", "x");

        // 2 字符中文 + 无扩展名：len=6，恰好命中 len-3 的中间位置
        assert!(generate("优化.md", "x").starts_with(SLUG_PREFIX));
    }

    /// `.md` 剥离必须大小写不敏感，否则 `.MD` 会把扩展名混进 kebab 段。
    #[test]
    fn extension_stripping_is_case_insensitive() {
        let lower = generate_parts("Note.md", "x");
        let upper = generate_parts("Note.MD", "x");
        let mixed = generate_parts("Note.Md", "x");

        assert_eq!(lower.kebab(), "note");
        assert_eq!(upper.kebab(), "note", ".MD 未被剥离：{}", upper.dir_name());
        assert_eq!(mixed.kebab(), "note", ".Md 未被剥离：{}", mixed.dir_name());

        // 无扩展名 / 非 .md 扩展名不应被误剥
        assert_eq!(generate_parts("Note", "x").kebab(), "note");
        assert_eq!(generate_parts("Note.markdown", "x").kebab(), "note-markdown");
        // 点号本身是分隔符，会折叠为横线
        assert_eq!(generate_parts("v1.2 说明.md", "x").kebab(), "v1-2");
    }

    /// 超长文件名：kebab 段截断后仍须合法（尾部横线要清掉、总长受控）。
    #[test]
    fn overlong_name_is_truncated_cleanly() {
        let slug = generate_parts("abcdefghij klmnopqrst uvwxyz 0123456789.md", "x");
        assert_eq!(slug.kebab().chars().count(), MAX_KEBAB_LEN);
        assert!(!slug.kebab().ends_with('-'));
        assert!(slug.dir_name().chars().count() <= 50);

        // 截断点正好落在横线上时，末尾横线必须被去掉
        let cut_on_dash = generate_parts("aaaaaaaaaa-bbbbbbbbbb-cccccccccc-ddddd.md", "x");
        assert!(
            !cut_on_dash.kebab().ends_with('-'),
            "截断处残留横线：{}",
            cut_on_dash.kebab()
        );
    }

    /// 同内容不同文件名 → hash 相同（内容寻址）、kebab 不同。
    #[test]
    fn hash_depends_on_content_not_filename() {
        let a = generate_parts("one.md", "相同内容");
        let b = generate_parts("two.md", "相同内容");
        assert_eq!(a.hash(), b.hash());
        assert_ne!(a.dir_name(), b.dir_name());
    }

    /// hash 段必须是小写十六进制，不能混入大写或非 hex 字符。
    #[test]
    fn hash_segment_is_lowercase_hex() {
        let slug = generate_parts("x.md", "任意内容");
        assert_eq!(slug.hash().len(), HASH_LEN);
        assert!(
            slug.hash()
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "hash 段含非小写 hex 字符：{}",
            slug.hash()
        );
    }

    /// Windows 保留设备名（`CON` / `NUL` / `COM1`…）不能成为目录名。
    /// 命名空间前缀天然规避了这一点——本测试锁定该性质，防止日后有人「优化」掉前缀。
    #[test]
    fn windows_reserved_names_are_neutralised_by_prefix() {
        for reserved in ["CON.md", "NUL.md", "AUX.md", "COM1.md", "LPT1.md", "PRN.md"] {
            let slug = generate(reserved, "x");
            assert!(
                slug.starts_with(SLUG_PREFIX),
                "「{reserved}」缺少前缀保护：{slug}"
            );
            // 前缀在前，完整目录名不可能是裸的保留名
            assert_ne!(slug.to_ascii_uppercase(), "CON");
        }
    }

    // ---------- 跨模块契约（防止 slug 与 Adapter 两侧分叉）----------

    /// **本任务最重要的一条测试**：`generate()` 的产出必须**永远**能通过
    /// `ensure_crossbrain_slug()` 的校验。
    ///
    /// 两者分处两个模块，各自演化时很容易错位：例如 slug 改用别的分隔符，
    /// 或校验规则收紧到「不许出现连续横线」——一旦分叉，
    /// 表现是**运行时** `sync_l2` 全部失败，而不是编译错误。
    /// 用 `SLUG_PREFIX` 单一常量 + 这条契约测试把两侧钉在一起。
    #[test]
    fn generated_slug_always_passes_adapter_namespace_check() {
        let filenames = [
            "Vue3 组件性能优化.md",
            "Rust · 生命周期与借用检查器.md",
            "C++ #include 踩坑记录.md",
            "优化.md",
            "../../etc/passwd.md",
            "a/b\\c.md",
            ".md",
            "",
            &"x".repeat(300),
        ];

        for filename in filenames {
            let slug = generate(filename, "内容");
            crate::adapters::ensure_crossbrain_slug(&slug).unwrap_or_else(|e| {
                panic!("slug「{slug}」（来自「{filename}」）未通过命名空间校验：{e}")
            });
        }
    }

    /// 命名空间前缀只能有一个定义：Adapter 侧引用的必须就是本模块这个常量。
    #[test]
    fn slug_prefix_single_source_of_truth() {
        assert_eq!(crate::adapters::SLUG_PREFIX, SLUG_PREFIX);
    }

    /// `skill_name()`（写入 frontmatter 的 `name`）与目录名的一致性：
    /// 前者必须等于目录名去掉前缀与 hash 段后的内容。
    #[test]
    fn skill_name_matches_directory_name_segments() {
        let slug = generate_parts("Vue3 组件性能优化.md", "x");
        let dir = slug.dir_name();

        assert_eq!(slug.skill_name(), slug.kebab());
        assert_eq!(
            dir,
            format!("{SLUG_PREFIX}{}-{}", slug.skill_name(), slug.hash()),
            "目录名必须能由「前缀 + 技能名 + hash」精确还原"
        );
        // frontmatter 的 name 不该带命名空间前缀（ADAPTER_SPEC 第 4 节示例：name: vue3-perf）
        assert!(
            !slug.skill_name().contains(SLUG_PREFIX),
            "技能名不应包含命名空间前缀：{}",
            slug.skill_name()
        );
    }

    /// `format_skill_md` 与本模块的衔接：用 `skill_name()` 作参数必须产出合法 frontmatter。
    #[test]
    fn skill_name_feeds_format_skill_md() {
        let slug = generate_parts("Vue3 组件性能优化.md", "x");
        let out = crate::adapters::format_skill_md(slug.skill_name(), "# 标题\n\n正文");

        assert!(
            out.starts_with("---\nname: vue3\n"),
            "frontmatter 未使用技能名：{out}"
        );
        assert!(out.contains("description: 标题"), "实际输出：{out}");
    }
}
