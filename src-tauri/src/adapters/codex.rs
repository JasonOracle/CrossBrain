//! codex.rs — Codex（OpenAI）的 Adapter 实现。
//!
//! # 「Codex」是桌面 App 与 CLI 的**同一个**配置目录
//!
//! 2026-09-14 实测：用户机器上装的是 **Codex 桌面应用**（MSIX 包身份 `OpenAI.Codex`，
//! 清单里的 DisplayName 就是 `Codex`，装在 `D:\soft\win-x64\`），该包自带
//! `resources/codex.exe`（CLI 本体），并把运行时下载到
//! `%LocalAppData%\OpenAI\Codex\{bin,runtimes}`。
//!
//! **桌面 App / 内置 CLI / Chrome 扩展三者共用同一个 Codex home `~/.codex/`**——
//! 官方文档亦确认「在桌面 App 里配置自定义指令，本质上也是写入 Codex home 的
//! AGENTS.md，与写入 `~/.codex/AGENTS.md` 等价」。
//!
//! 故本 Adapter **只有一个落点、一个显示名 `"Codex"`**，不为「App」「CLI」各做一套
//! ——它们本就写同一个文件，分开做只会变成同一份内容写两遍。
//!
//! （历史称呼：本任务立项时记为「Codex CLI」，2026-09-14 探测到桌面 App 后更名为
//! `Codex`。这与 PRD 原先误记的「ChatGPT IDE」是同一类命名错误的第二次修正。）
//!
//! # 目标路径（2026-09-14 本机预探测，详见 `docs/testing/SPIKE_RESULTS.md`）
//!
//! | 用途 | 路径 |
//! |:---|:---|
//! | 安装检测 | `~/.codex/` |
//! | L0 全局规则 | `~/.codex/AGENTS.md` —— ⚠️ **标记块协议 + 内联规则全文** |
//! | L2 技能知识 | `~/.codex/skills/{slug-hash}/SKILL.md` |
//! | 孤儿清理 | `~/.codex/skills/` 下的 `crossbrain-*` 目录 |
//! | 备份 | `~/.codex/AGENTS.md.crossbrain-backup` |
//!
//! # ⚠️ `AGENTS.override.md` 会让我们**完全失效**（同步前必须查）
//!
//! Codex 的规则发现顺序是：**先看 `AGENTS.override.md`，没有再回落到 `AGENTS.md`**，
//! 且**每个层级只取第一个非空文件**。所以只要 `~/.codex/AGENTS.override.md` 存在，
//! 我们把标记块写进 `AGENTS.md` 就**永远不会被 Codex 读到**——典型的静默失效
//! （无报错、无效果）。处理方式见 [`CodexAdapter::reject_if_override_present`]。
//!
//! # 为什么 L0 必须走标记块注入（不能像 Antigravity 那样写独立文件）
//!
//! Antigravity 的 `~/.gemini/config/rules/` 是「整个目录下所有 `.md` 都被读取」，
//! 所以 CrossBrain 能写独立文件 `crossbrain-L0.md`，与用户文件物理隔离。
//!
//! **Codex 不是这样：它只读 `~/.codex/AGENTS.md` 这一个文件。** 若照搬 Antigravity
//! 写 `crossbrain-L0.md`，该文件**永远不会被 Codex 读到**——同步静默失效，
//! 无报错、无效果，属最难排查的一类问题。
//!
//! # 为什么标记块内容是内联，而不是 Claude 那样的 `@` 引用
//!
//! Claude 侧标记块恒为一行 `@~/.ai-profile/AGENTS.md`。`@` 是 **Claude Code 专有**
//! 的路径展开语法（`ADAPTER_SPEC.md` 已明确），Spike A 只验证过它在 Claude 中可用。
//! **在 Codex 中能否展开从未验证，且没有理由认为可以。**
//!
//! 故本 Adapter 直接内联 `rules_content` 全文：
//!
//! ```text
//! <!-- CrossBrain:Start -->
//! <!-- 本区域由 CrossBrain 自动维护，修改请在 CrossBrain App 中进行 -->
//! （global/rules.md 的完整内容）
//! <!-- CrossBrain:End -->
//! ```
//!
//! 代价是规则内容在 Codex 侧各存一份（Claude 侧只有引用）。这个代价可以接受：
//! CrossBrain 的职责本就是「把 SSOT 分发到各工具」，分发意味着每个工具
//! 都拿到**自己能读懂**的形态。
//!
//! # ⛔ 严禁触碰的路径
//!
//! | 路径 | 是什么 | 后果 |
//! |:---|:---|:---|
//! | `~/.codex/skills/.system/` | Codex **内置技能** | 删掉会破坏 Codex 本体功能 |
//! | `~/.codex/rules/default.rules` | **命令审批白名单**，不是规则文件 | 写坏 Codex 的沙箱策略 |
//! | `~/.codex/memories/` | Codex 自己的记忆机制 | 工具内部数据 |
//! | `~/.codex/config.toml` | Codex 主配置 | 同上 |
//! | `~/.codex/AGENTS.override.md` | 用户的**覆盖指令**，优先级高于 `AGENTS.md` | 本 Adapter **不写**它；它一旦存在，同步直接中止并报错 |
//!
//! > `cleanup_crossbrain_orphans()` 只处理 `SLUG_PREFIX` 前缀的目录，
//! > 所以 `.system` 因前缀不匹配被自动跳过——这是**结构性安全**，
//! > 不依赖「记得排除它」这种人的注意力。
//!
//! # 铁律
//!
//! - **L-01**：标记块格式逐字符固定，且永不整体覆盖用户文件
//! - **ADR-09**：孤儿清理只允许删除 `crossbrain-` 前缀的目录
//! - **L-02**：路径一律取自 [`crate::paths`]，不得自行拼接或硬编码盘符

use std::fs;
use std::path::PathBuf;

use super::{
    cleanup_crossbrain_orphans, delete_backup_files, ensure_crossbrain_slug,
    ensure_no_marker_in_content, inject_marker_block, pre_restore_path,
    remove_marker_block_from_file, skill_cleanup_actions, Adapter, AdapterError, CleanupReport,
    L0Backup, MARKER_END, MARKER_NOTE, MARKER_START,
};
use crate::paths;

/// 转出共享结果类型，使 `adapters::codex::SyncL0Result` 与其它 Adapter 路径对称。
pub use super::SyncL0Result;

/// 备份后缀的本地别名（真实定义在共享层 [`super::MARKER_BACKUP_SUFFIX`]）。
use super::MARKER_BACKUP_SUFFIX as BACKUP_SUFFIX;

/// 目标文件名。
const AGENTS_MD_NAME: &str = "AGENTS.md";

/// 会**遮蔽** `AGENTS.md` 的覆盖文件名（优先级更高，同步前必须查）。
const AGENTS_OVERRIDE_NAME: &str = "AGENTS.override.md";

/// 技能文件名（三个 Adapter 通用，见 `ADAPTER_SPEC.md` 4.1）。
const SKILL_FILE_NAME: &str = "SKILL.md";

/// 技能根目录名。
const SKILLS_DIR_NAME: &str = "skills";

/// Codex 适配器。
///
/// # 关于 `base_dir`
///
/// 生产路径由 [`crate::paths::codex_dir`] 解析为 `~/.codex/`。
/// 但**测试绝不能在真实目录上跑**：这里执行的是真实写入，会改动用户正在使用的
/// `~/.codex/AGENTS.md`（Codex 每次会话都会读它）。
///
/// 因此提供 [`CodexAdapter::with_base_dir`] 把根目录重定向到临时目录，
/// 让标记块协议与内联行为都能在完全隔离的沙箱里被验证。
#[derive(Debug, Clone, Default)]
pub struct CodexAdapter {
    /// 配置根目录覆盖；`None` = 使用真实的 `~/.codex/`。
    base_dir: Option<PathBuf>,
}

impl CodexAdapter {
    /// 生产用构造：指向真实的 `~/.codex/`。
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造一个把所有读写重定向到 `base_dir` 的实例。
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// 配置根目录。
    fn codex_dir_inner(&self) -> PathBuf {
        match &self.base_dir {
            Some(dir) => dir.clone(),
            None => paths::codex_dir(),
        }
    }

    /// `AGENTS.md` 完整路径。
    fn agents_md_file(&self) -> PathBuf {
        self.codex_dir_inner().join(AGENTS_MD_NAME)
    }

    /// `AGENTS.override.md` 完整路径。
    ///
    /// ⚠️ 本 Adapter **只读检测**它，**永不写入**——它是用户的覆盖指令，
    /// 优先级高于我们托管的 `AGENTS.md`。
    fn agents_override_file(&self) -> PathBuf {
        self.codex_dir_inner().join(AGENTS_OVERRIDE_NAME)
    }

    /// 备份文件路径（`AGENTS.md.crossbrain-backup`）。
    fn backup_file(&self) -> PathBuf {
        self.codex_dir_inner()
            .join(format!("{AGENTS_MD_NAME}{BACKUP_SUFFIX}"))
    }

    /// `skills/` 目录。
    fn skills_dir(&self) -> PathBuf {
        self.codex_dir_inner().join(SKILLS_DIR_NAME)
    }

    /// 指定技能的 `SKILL.md` 路径。
    fn skill_file(&self, slug_hash: &str) -> PathBuf {
        self.skills_dir().join(slug_hash).join(SKILL_FILE_NAME)
    }

    /// 构造 Codex 的标记块：**内联规则全文**（与 Claude 的 `@` 引用不同）。
    ///
    /// # 为什么返回 `Result`（内联形态特有的校验点）
    ///
    /// 内联内容来自用户的 `global/rules.md`。若里面出现了 `MARKER_END`
    /// （哪怕只是「举例说明标记块长什么样」），标记块的非贪婪正则会**在错误的
    /// 位置提前闭合**：本次写入看似成功，下次同步却会重写错误的区间。
    ///
    /// 故写入前先检查，命中则报错中止（ADR-14「失败可见」原则）——
    /// 宁可让用户看到一次明确的失败，也不能静默产出一个已损坏的文件。
    ///
    /// Claude 侧不需要这个校验：它的标记块内容恒为固定的一行引用，不含用户输入。
    fn build_marker_block(rules_content: &str) -> Result<String, AdapterError> {
        ensure_no_marker_in_content(rules_content)?;

        let body = rules_content.trim_end();
        Ok(if body.is_empty() {
            // 规则为空时不留下多余空行（正常流程里空规则会被上层跳过，
            // 这里只是保证 Adapter 单独被调用时也产出干净结果）
            format!("{MARKER_START}\n{MARKER_NOTE}\n{MARKER_END}")
        } else {
            format!("{MARKER_START}\n{MARKER_NOTE}\n{body}\n{MARKER_END}")
        })
    }

    /// `AGENTS.override.md` 存在即中止本次 L0 同步。
    ///
    /// # 为什么必须显式检查（而不是让它自然失效）
    ///
    /// Codex 的规则发现顺序是「先 `AGENTS.override.md`，没有再回落 `AGENTS.md`」，
    /// 且每个层级**只取第一个非空文件**。所以该文件存在时，我们写进 `AGENTS.md`
    /// 的标记块永远不会被读到——**同步会「成功」但毫无效果**，
    /// 属本项目最难排查的一类问题（见 `MEMORY.md` 的「三 Adapter 形态」节）。
    ///
    /// # 为什么是「中止并报错」而不是别的处理
    ///
    /// - ❌ 静默写入 `AGENTS.md`：用户看不到任何异常，但规则就是不生效
    /// - ❌ 改写到 `AGENTS.override.md`：用户日后删掉该文件时，
    ///   `AGENTS.md` 里会留下一块陈旧标记块需要善后，语义复杂且易错
    /// - ✅ 明确报错：符合 ADR-14「失败可见、可重试」——宁可让用户看到一次
    ///   清晰的失败，也不能静默产出「看似成功、实则无效」的结果
    ///
    /// # 为什么不区分「空文件」
    ///
    /// Codex 会跳过空文件，所以「存在但 0 字节」的 override 理论上不遮蔽我们。
    /// 但**不特判**：① 该行为是 Codex 的实现细节，可能变化；
    /// ② 空 override 恰恰说明用户正打算用它，此刻放行等于给将来埋一颗
    /// 「某天用户往里写一行 → 我们的规则无声失效」的雷。
    /// 统一按「存在即中止」处理，规则单一、易于解释。
    fn reject_if_override_present(&self) -> Result<(), AdapterError> {
        let override_file = self.agents_override_file();
        if override_file.exists() {
            return Err(AdapterError::Other(format!(
                "检测到 {} 。Codex 的规则读取顺序是「先 AGENTS.override.md、\
                 再回落到 AGENTS.md，且只取第一个非空文件」，因此只要该文件存在，\
                 CrossBrain 写入 AGENTS.md 的规则就会被 Codex 完全忽略。\
                 请删除或重命名它后重新同步。",
                override_file.display()
            )));
        }
        Ok(())
    }

    /// 执行四情况协议，返回是否需要弹 UI 提示。
    ///
    /// 控制流本体在共享层 [`super::inject_marker_block`]，与 Claude Code Adapter
    /// 完全共用——两者的差异只有目标文件、备份文件、标记块内容三项。
    ///
    /// # 幂等性
    ///
    /// [`Self::build_marker_block`] 是纯函数（同输入 → 同输出），
    /// 故情况二的「重新生成并替换」不会累积内容：同内容连续同步多次，
    /// 文件字节完全一致。
    pub fn sync_l0_with_result(&self, rules_content: &str) -> Result<SyncL0Result, AdapterError> {
        // ① 先查遮蔽文件——否则后面所有工作都可能白做（写进去没人读）
        self.reject_if_override_present()?;
        // ② 再校验内联内容不会破坏标记块
        let marker_block = Self::build_marker_block(rules_content)?;
        // ③ 最后走共享的四情况协议
        inject_marker_block(&self.agents_md_file(), &self.backup_file(), &marker_block)
    }

    // 断链写入（D-03）与四情况协议均由共享层提供，本文件不保留任何副本：
    // `super::write_breaking_hardlink` + `super::inject_marker_block`。
    // 本机 `~/.codex/AGENTS.md` 当前 `links=1`，但用户随时可能像对待
    // `CLAUDE.md` 那样给它建硬链接——共用同一条写入通道即可自动获得保护，
    // 该行为已被 `sync_l0_does_not_pollute_hardlinked_sibling` 测试锁住。
}

impl Adapter for CodexAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(self.codex_dir_inner().is_dir())
    }

    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> {
        // trait 契约只需要「成功/失败」，四情况的具体分支结果由
        // `sync_l0_with_result` 承载给 UI（情况三需要弹一次提示）。
        self.sync_l0_with_result(rules_content).map(|_| ())
    }

    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError> {
        ensure_crossbrain_slug(slug_hash)?;

        let file = self.skill_file(slug_hash);
        // 路径末段固定为 SKILL_FILE_NAME，父目录必然存在
        let dir = file
            .parent()
            .ok_or_else(|| AdapterError::Other(format!("非法路径：{}", file.display())))?;
        fs::create_dir_all(dir)
            .map_err(|e| AdapterError::DirectoryCreateFailed(format!("{}：{e}", dir.display())))?;

        // SKILL.md 是 CrossBrain 全权托管的新文件，不存在硬链接共享问题，
        // 也无需保护用户内容 → 直接写（与 Claude / Antigravity 侧一致）。
        fs::write(&file, content.as_bytes())
            .map_err(|e| AdapterError::WriteError(format!("{}：{e}", file.display())))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        // 清理规则与另外两个 Adapter 完全一致，故共用 `super::cleanup_crossbrain_orphans`；
        // 差异只在 skills_dir 的取值。
        //
        // ⚠️ `~/.codex/skills/.system/`（Codex 内置技能）因不带 `crossbrain-` 前缀
        // 被该函数自动跳过——**不要**为此加特例判断，那会把结构性安全降级成
        // 「依赖有人记得排除它」。
        cleanup_crossbrain_orphans(&self.skills_dir(), active_slugs)
    }

    fn skills_root(&self) -> PathBuf {
        self.skills_dir()
    }

    fn l0_backup(&self) -> Option<L0Backup> {
        // Codex 同样是**标记块注入型**：我们改的是用户的 `~/.codex/AGENTS.md` 本身，
        // 首次注入前留了 `.crossbrain-backup`（TASK-19 / ADR-15）。
        //
        // 唯一走 `None` 的是 Antigravity——它写自己的独立文件，从不碰用户既有文件。
        Some(L0Backup {
            target_file: self.agents_md_file(),
            backup_file: self.backup_file(),
        })
    }

    fn uninstall(&self) -> Result<Vec<String>, AdapterError> {
        let mut actions = Vec::new();

        // ① 移除标记块（内联的规则全文随之消失，用户自己的指令原样保留）。
        //    顺序与 Claude Code 相同：先移块、后删备份。
        if let Some(desc) =
            remove_marker_block_from_file(&self.agents_md_file(), &self.backup_file())?
        {
            actions.push(desc);
        }

        // ② 删除备份文件（去痕）
        actions.extend(delete_backup_files(
            &self.backup_file(),
            &pre_restore_path(&self.agents_md_file()),
        ));

        // ③ 清空 skills 目录下的 crossbrain-* 目录（`skills/.system/` 不带前缀，
        //    由共享清理函数的结构性安全自动跳过）
        actions.extend(skill_cleanup_actions(self.cleanup_orphans(&[])?));

        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    //! 所有测试都在**临时目录**里跑（通过 `with_base_dir` 重定向），
    //! 绝不触碰真实的 `~/.codex/AGENTS.md` ——那里是用户正在使用的全局指令文件。

    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 用「进程号 + 计数器」命名，不依赖时钟，且进程内互不冲突。
    static SEQ: AtomicU32 = AtomicU32::new(0);

    const USER_ORIGINAL: &str = "# 我的 Codex 全局指令\n\n- 用中文回答\n";

    /// 隔离的临时配置根目录，`Drop` 时自动清理。
    struct TempEnv {
        root: PathBuf,
    }

    impl TempEnv {
        fn new(tag: &str) -> Self {
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "crossbrain-codex-{}-{tag}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).expect("创建临时测试根目录失败");
            Self { root }
        }

        fn adapter(&self) -> CodexAdapter {
            CodexAdapter::with_base_dir(&self.root)
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.root.join(rel)
        }

        fn agents_md(&self) -> PathBuf {
            self.root.join(AGENTS_MD_NAME)
        }

        fn backup(&self) -> PathBuf {
            self.root.join(format!("{AGENTS_MD_NAME}{BACKUP_SUFFIX}"))
        }

        fn override_file(&self) -> PathBuf {
            self.root.join(AGENTS_OVERRIDE_NAME)
        }
    }

    impl Drop for TempEnv {
        fn drop(&mut self) {
            // Windows 上文件句柄可能延迟释放（本机实测过 os error 5）；
            // 清理失败不影响测试结论，忽略即可。
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    // ---------- detect ----------

    #[test]
    fn detect_reflects_codex_dir_existence() {
        let env = TempEnv::new("detect");
        let adapter = env.adapter();

        assert!(adapter.detect().unwrap(), "配置目录存在时 → true");

        fs::remove_dir_all(&env.root).unwrap();
        assert!(!adapter.detect().unwrap(), "配置目录不存在时 → false");
    }

    // ---------- 情况一 ----------

    #[test]
    fn case1_creates_file_with_marker_block_when_missing() {
        let env = TempEnv::new("case1");
        let result = env.adapter().sync_l0_with_result("用中文回答").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况一必须静默，不弹提示");

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.starts_with(MARKER_START), "文件应以标记块开头：{content}");
        assert!(content.ends_with(MARKER_END), "文件应以结束标志收尾：{content}");
        assert_eq!(
            content,
            format!("{MARKER_START}\n{MARKER_NOTE}\n用中文回答\n{MARKER_END}"),
            "标记块格式必须逐字符固定"
        );
    }

    /// 内联是 Codex 与 Claude 的**核心差异**：规则全文必须在块内，
    /// 而不是一行 `@` 引用。若这里退化成引用，同步会静默失效。
    #[test]
    fn case1_inlines_full_rules_content_not_a_reference() {
        let env = TempEnv::new("inline");
        let rules = "# 全局规则\n\n- 先给结论\n- 不确定就说不确定";
        env.adapter().sync_l0_with_result(rules).unwrap();

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.contains(rules), "规则全文未被内联：\n{content}");
        assert!(
            !content.contains("@~/.ai-profile/AGENTS.md"),
            "出现了 Claude 专有的 @ 引用语法——Codex 不认识它，会导致同步静默失效"
        );
    }

    #[test]
    fn case1_then_case2_repeated_call_is_idempotent() {
        let env = TempEnv::new("case1-idem");
        let adapter = env.adapter();

        adapter.sync_l0_with_result("规则内容").unwrap();
        let first = fs::read_to_string(env.agents_md()).unwrap();

        // 第二次进来已是情况二（文件已含标记块），结果必须完全相同
        let result = adapter.sync_l0_with_result("规则内容").unwrap();
        assert_eq!(result, SyncL0Result::Silent);
        assert_eq!(
            fs::read_to_string(env.agents_md()).unwrap(),
            first,
            "重复同步不得累积标记块"
        );
    }

    #[test]
    fn case1_creates_missing_parent_directory() {
        let env = TempEnv::new("mkdir");
        let nested = env.path("nested/deeper");
        let adapter = CodexAdapter::with_base_dir(&nested);

        adapter.sync_l0_with_result("规则").unwrap();
        assert!(
            nested.join(AGENTS_MD_NAME).is_file(),
            "父目录不存在时应自动创建"
        );
    }

    // ---------- 情况二 ----------

    #[test]
    fn case2_replaces_marker_block_and_preserves_user_content() {
        let env = TempEnv::new("case2");
        let with_block = format!(
            "{}\n\n{}\n{MARKER_NOTE}\n旧规则\n{MARKER_END}",
            USER_ORIGINAL.trim_end(),
            MARKER_START
        );
        fs::write(env.agents_md(), &with_block).unwrap();

        let result = env.adapter().sync_l0_with_result("新规则").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况二必须静默");
        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.contains("新规则"), "块内内容未更新：{content}");
        assert!(!content.contains("旧规则"), "块内旧内容未被替换：{content}");
        assert!(
            content.contains("用中文回答"),
            "标记块之外的用户内容被破坏：{content}"
        );
    }

    /// 「只有 Start 没有 End」不得被误判为情况二。
    ///
    /// 这正是原 Claude 实现踩过的坑：`contains(Start) && contains(End)` 判真后，
    /// 正则匹配不到、替换静默失败，于是返回成功但文件没改。
    #[test]
    fn case2_detection_rejects_unclosed_marker() {
        let env = TempEnv::new("unclosed");
        let broken = format!("{USER_ORIGINAL}\n{MARKER_START}\n没有结束标志\n");
        fs::write(env.agents_md(), &broken).unwrap();

        let result = env.adapter().sync_l0_with_result("规则").unwrap();

        // 判定必须落到情况三（备份 + 追加），而不是情况二
        assert!(
            matches!(result, SyncL0Result::BackupCreated { .. }),
            "未闭合的标记块被误判为情况二，实际：{result:?}"
        );
        assert!(env.backup().is_file(), "应已创建备份");
    }

    // ---------- 情况三 / 四 ----------

    #[test]
    fn case3_backs_up_then_appends_and_reports() {
        let env = TempEnv::new("case3");
        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();

        let result = env.adapter().sync_l0_with_result("规则").unwrap();

        assert_eq!(
            result,
            SyncL0Result::BackupCreated {
                backup_path: env.backup().to_string_lossy().to_string()
            },
            "情况三必须返回备份路径，供 UI 提示"
        );

        // 备份内容 == 改动前的原文（顺序错了就会备份到改后的内容）
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            USER_ORIGINAL,
            "备份内容与原始内容不一致"
        );

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.contains("用中文回答"), "原有内容未保留：{content}");
        assert!(content.contains(MARKER_START), "标记块未追加：{content}");
        assert!(
            content.find(MARKER_START).unwrap() > content.find("用中文回答").unwrap(),
            "标记块应追加在文末而非插到用户内容之前"
        );
    }

    #[test]
    fn case4_does_not_overwrite_existing_backup() {
        let env = TempEnv::new("case4");
        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        // 预置一份「更早的干净快照」——情况四绝不能覆盖它
        fs::write(env.backup(), "更早的原始快照").unwrap();

        let result = env.adapter().sync_l0_with_result("规则").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况四不应重复弹提示");
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            "更早的原始快照",
            "旧备份被覆盖了——用户就此失去原始快照"
        );
        assert!(fs::read_to_string(env.agents_md())
            .unwrap()
            .contains(MARKER_START));
    }

    // ---------- 防注入 ----------

    #[test]
    fn rejects_rules_content_containing_marker_end() {
        let env = TempEnv::new("inject");
        let evil = format!("普通规则\n\n{MARKER_END}\n\n更多内容");

        let err = env
            .adapter()
            .sync_l0_with_result(&evil)
            .expect_err("含结束标记的内容必须被拒绝");

        let msg = err.to_string();
        assert!(msg.contains("结束标记"), "错误信息应指明原因：{msg}");
        assert!(
            !env.agents_md().exists(),
            "被拒绝的写入不得留下任何文件（避免半成品状态）"
        );
    }

    #[test]
    fn rejects_rules_content_containing_marker_start() {
        let env = TempEnv::new("inject-start");
        let evil = format!("规则\n{MARKER_START}\n");
        assert!(
            env.adapter().sync_l0_with_result(&evil).is_err(),
            "含起始标记的内容同样必须被拒绝"
        );
    }

    /// 被拒绝后原文件必须**一字未动**——这是「拒绝」与「损坏」的分界。
    #[test]
    fn rejection_leaves_existing_file_untouched() {
        let env = TempEnv::new("inject-safe");
        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        let before = fs::read_to_string(env.agents_md()).unwrap();

        let evil = format!("x\n{MARKER_END}");
        assert!(env.adapter().sync_l0_with_result(&evil).is_err());

        assert_eq!(
            fs::read_to_string(env.agents_md()).unwrap(),
            before,
            "拒绝写入时原文件被改动了"
        );
        assert!(!env.backup().exists(), "拒绝路径不应留下备份");
    }

    // ---------- AGENTS.override.md 遮蔽检测 ----------

    /// 核心用例：override 文件存在时，L0 同步必须**明确失败**，
    /// 而不是「成功但规则没人读」。
    #[test]
    fn override_file_blocks_l0_sync_with_visible_error() {
        let env = TempEnv::new("override");
        fs::write(env.override_file(), "# 用户的覆盖指令\n- 用英文\n").unwrap();

        let err = env
            .adapter()
            .sync_l0_with_result("规则")
            .expect_err("override 存在时必须中止，否则规则会被静默忽略");

        let msg = err.to_string();
        assert!(
            msg.contains(AGENTS_OVERRIDE_NAME),
            "错误信息应点明是哪个文件导致的：{msg}"
        );
        assert!(
            !env.agents_md().exists(),
            "被拒绝的同步不得留下半个 AGENTS.md"
        );
        assert!(!env.backup().exists(), "被拒绝的同步不应创建备份");
    }

    /// 0 字节的 override 同样中止。
    ///
    /// Codex 会跳过空文件，所以这是**故意从严**：空 override 说明用户正准备用它，
    /// 此刻放行等于给将来埋一颗「某天往里写一行 → 规则无声失效」的雷。
    /// 规则单一（存在即中止）比「精确模拟 Codex 的空文件语义」更易解释、更安全。
    #[test]
    fn override_file_blocks_even_when_empty() {
        let env = TempEnv::new("override-empty");
        fs::write(env.override_file(), "").unwrap();

        assert!(
            env.adapter().sync_l0_with_result("规则").is_err(),
            "空 override 也应中止（有意从严，见本测试注释）"
        );
    }

    /// 被遮蔽而中止时，用户既有的 `AGENTS.md` 必须**一字未动**。
    #[test]
    fn override_rejection_leaves_existing_agents_md_untouched() {
        let env = TempEnv::new("override-safe");
        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        fs::write(env.override_file(), "# 覆盖\n").unwrap();

        assert!(env.adapter().sync_l0_with_result("规则").is_err());
        assert_eq!(
            fs::read_to_string(env.agents_md()).unwrap(),
            USER_ORIGINAL,
            "中止路径不得改动用户既有的 AGENTS.md"
        );
    }

    /// 反面对照：没有 override 时同步照常成功——确保上面三条测的是
    /// 「override 的遮蔽」而不是「同步本身坏了」。
    #[test]
    fn l0_sync_proceeds_when_no_override_present() {
        let env = TempEnv::new("no-override");
        assert!(!env.override_file().exists());

        let result = env.adapter().sync_l0_with_result("规则").unwrap();
        assert_eq!(result, SyncL0Result::Silent);
        assert!(fs::read_to_string(env.agents_md())
            .unwrap()
            .contains(MARKER_START));
    }

    // ---------- 硬链接隔离 ----------

    /// 若用户像对待 `CLAUDE.md` 那样给 `AGENTS.md` 建了硬链接，
    /// 写入必须只影响 `AGENTS.md` 自己，不牵动兄弟路径（D-03 同一条约束）。
    #[test]
    fn sync_l0_does_not_pollute_hardlinked_sibling() {
        let env = TempEnv::new("hardlink");
        let sibling = env.path("opencode_AGENTS.md");

        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        fs::hard_link(env.agents_md(), &sibling).expect("创建硬链接失败（需 NTFS）");

        env.adapter().sync_l0_with_result("规则").unwrap();

        assert_eq!(
            fs::read_to_string(&sibling).unwrap(),
            USER_ORIGINAL,
            "兄弟路径被污染——说明写入没有断开硬链接"
        );
        assert!(fs::read_to_string(env.agents_md())
            .unwrap()
            .contains(MARKER_START));
    }

    // ---------- sync_l2 ----------

    #[test]
    fn sync_l2_writes_skill_file() {
        let env = TempEnv::new("l2");
        let adapter = env.adapter();
        let slug = "crossbrain-vue3-9f86d0";
        let body = "---\nname: vue3\ndescription: Vue3 性能\n---\n\n# Vue3\n";

        adapter.sync_l2(slug, body).unwrap();

        let file = env.path(&format!("{SKILLS_DIR_NAME}/{slug}/{SKILL_FILE_NAME}"));
        assert!(file.is_file(), "SKILL.md 未创建：{}", file.display());
        assert_eq!(fs::read_to_string(&file).unwrap(), body, "内容必须原样落盘");

        // 幂等：重复写入不报错、内容不变
        adapter.sync_l2(slug, body).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), body);
    }

    #[test]
    fn sync_l2_rejects_slug_outside_namespace() {
        let env = TempEnv::new("l2-reject");
        let adapter = env.adapter();

        // 用户自建技能名——绝不能覆盖
        assert!(adapter.sync_l2("grill-me", "x").is_err(), "未拦截用户技能名");
        // 路径上跳——绝不能写到 skills/ 之外
        assert!(
            adapter.sync_l2("crossbrain-../../evil", "x").is_err(),
            "未拦截路径上跳"
        );
        assert!(
            !env.path("evil").exists(),
            "非法 slug 竟然在 skills 目录之外产生了文件"
        );
    }

    // ---------- 孤儿清理 ----------

    #[test]
    fn cleanup_removes_orphans_and_protects_non_crossbrain_content() {
        let env = TempEnv::new("cleanup");
        let skills = env.path(SKILLS_DIR_NAME);

        // 模拟真实 `~/.codex/skills/` 的四类内容
        fs::create_dir_all(skills.join(".system/imagegen")).unwrap(); // Codex 内置（⛔）
        fs::create_dir_all(skills.join("grill-me")).unwrap(); // 用户自建
        fs::create_dir_all(skills.join("crossbrain-old-000000")).unwrap(); // 孤儿
        fs::create_dir_all(skills.join("crossbrain-keep-111111")).unwrap(); // 本次活跃

        let active = vec!["crossbrain-keep-111111".to_string()];
        let report = env.adapter().cleanup_orphans(&active).unwrap();

        assert_eq!(
            report.deleted_dirs,
            vec!["crossbrain-old-000000".to_string()],
            "只应删除孤儿"
        );
        assert_eq!(
            report.kept_dirs,
            vec!["crossbrain-keep-111111".to_string()]
        );

        // 最不能出错的两项：Codex 内置技能与用户技能必须完好
        assert!(skills.join(".system/imagegen").is_dir(), "Codex 内置技能被删了");
        assert!(skills.join("grill-me").is_dir(), "用户自建技能被删了");
        assert!(!skills.join("crossbrain-old-000000").exists());
        assert!(skills.join("crossbrain-keep-111111").is_dir());
    }

    /// skills 目录不存在 = 从未同步过，属正常情况而非错误。
    #[test]
    fn cleanup_is_noop_when_skills_dir_missing() {
        let env = TempEnv::new("cleanup-missing");
        let report = env.adapter().cleanup_orphans(&[]).unwrap();
        assert!(report.deleted_dirs.is_empty());
        assert!(report.kept_dirs.is_empty());
    }

    /// 同名普通文件（非目录）不得被当成孤儿删除。
    #[test]
    fn cleanup_skips_plain_files_even_with_prefix() {
        let env = TempEnv::new("cleanup-file");
        let skills = env.path(SKILLS_DIR_NAME);
        fs::create_dir_all(&skills).unwrap();
        let decoy = skills.join("crossbrain-not-a-dir");
        fs::write(&decoy, "我是文件，不是目录").unwrap();

        let report = env.adapter().cleanup_orphans(&[]).unwrap();

        assert!(report.deleted_dirs.is_empty(), "普通文件不应进入删除清单");
        assert!(decoy.is_file(), "普通文件被误删了");
    }

    // ---------- trait 契约一致性 ----------

    /// trait 的 `sync_l0` 丢弃分支结果，行为必须与 `sync_l0_with_result` 等价。
    #[test]
    fn trait_sync_l0_matches_with_result_variant() {
        let a = TempEnv::new("trait-a");
        let b = TempEnv::new("trait-b");
        fs::write(a.agents_md(), USER_ORIGINAL).unwrap();
        fs::write(b.agents_md(), USER_ORIGINAL).unwrap();

        a.adapter().sync_l0("规则").unwrap();
        b.adapter().sync_l0_with_result("规则").unwrap();

        assert_eq!(
            fs::read_to_string(a.agents_md()).unwrap(),
            fs::read_to_string(b.agents_md()).unwrap(),
            "两条入口产出的文件内容必须一致"
        );
    }

    /// 卸载全周期：内联注入 → 卸载 → 用户指令原样回来、备份与技能目录清空（TASK-14）。
    #[test]
    fn uninstall_restores_user_content_and_removes_traces() {
        let env = TempEnv::new("uninstall");
        let adapter = env.adapter();

        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        let skill = env.root.join("skills").join("crossbrain-demo-abc123");
        fs::create_dir_all(&skill).unwrap();
        let system = env.root.join("skills").join(".system");
        fs::create_dir_all(&system).unwrap();

        adapter.sync_l0("规则正文").unwrap();
        assert!(env.backup().is_file());

        let actions = adapter.uninstall().unwrap();
        assert!(
            actions.iter().any(|a| a.contains("原样保留")),
            "动作描述要向用户说明内容未丢：{actions:?}"
        );

        let restored = fs::read_to_string(env.agents_md()).unwrap();
        assert_eq!(restored, USER_ORIGINAL, "移除后必须回到用户原文");
        assert!(!env.backup().exists(), "备份必须被删除");
        assert!(!skill.exists(), "crossbrain-* 技能目录必须被删除");
        assert!(system.exists(), "skills/.system/（Codex 内置）绝不能被删");
    }
}
