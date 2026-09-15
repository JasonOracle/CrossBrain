//! claude_code.rs — Claude Code 的 Adapter 实现。
//!
//! # 目标路径（依据 `docs/tech/ADAPTER_SPEC.md` 第 3 节）
//!
//! | 用途 | 路径 |
//! |:---|:---|
//! | 安装检测 | `~/.claude/` |
//! | L0 全局规则 | `~/.claude/CLAUDE.md` —— ⚠️ **标记块协议**（铁律 L-01） |
//! | L2 技能知识 | `~/.claude/skills/{slug-hash}/SKILL.md` |
//! | 孤儿清理 | `~/.claude/skills/` 下的 `crossbrain-*` 目录 |
//! | 备份 | `~/.claude/CLAUDE.md.crossbrain-backup` |
//!
//! # 本 Adapter 是唯一不可「全量覆盖」的实现
//!
//! `CLAUDE.md` 里往往有用户自己写的大量规则，整体覆盖 = 不可逆的数据丢失。
//! 因此必须走**四情况协议**：只在标记块内改动，标记块之外的用户内容一字不动。
//!
//! # 决策 D-03：写入必须「断开硬链接」
//!
//! 这是本项目最反直觉的一条约束。本机实测（`fsutil hardlink list`，links=5）：
//! 用户把同一份全局规则做成了**硬链接**，5 个路径共享同一个 inode：
//!
//! ```text
//! ~/.ai-memory/user_profile.md            ← 原始
//! ~/.cursor/rules/user_profile.md
//! ~/.config/opencode/AGENTS.md
//! ~/.gemini/config/rules/user_global.md
//! ~/.claude/CLAUDE.md                     ← 本 Adapter 的目标
//! ```
//!
//! **若就地覆写**：内容会同时出现在全部 5 个工具里——CrossBrain 的标记块会被
//! Cursor / OpenCode / Antigravity 也读到，而 `@~/.ai-profile/AGENTS.md` 是
//! Claude 专属的引用语法，在其它工具中毫无意义，属于跨工具污染。
//!
//! **因此**：本 Adapter 对 `CLAUDE.md` 的**所有**写入（情况一~四，无一例外）
//! 一律经 [`write_breaking_hardlink`]——先删原路径再落盘，强制让 `CLAUDE.md`
//! 变成独立文件，与其余 4 个工具彻底隔离。
//!
//! 该取舍由用户拍板（`CURRENT_STATUS.md` D-03：断开硬链接 / 原子替换），
//! 即便代价是用户原本「一处修改、五处生效」的机制在 Claude 这一侧被打破。
//!
//! # 铁律
//!
//! - **L-01**：标记块格式逐字符固定，且永不整体覆盖用户文件
//! - **ADR-09**：孤儿清理只允许删除 `crossbrain-` 前缀的目录
//! - **L-02**：路径一律取自 [`crate::paths`]，不得自行拼接或硬编码盘符

use std::fs;
use std::path::PathBuf;

use super::{
    cleanup_crossbrain_orphans, delete_backup_files, ensure_crossbrain_slug, inject_marker_block,
    pre_restore_path, remove_marker_block_from_file, skill_cleanup_actions, Adapter, AdapterError,
    CleanupReport, L0Backup,
};
use crate::paths;

// ---------------------------------------------------------------------------
// 标记块协议已提升到共享层（`adapters/mod.rs`）
// ---------------------------------------------------------------------------
//
// Claude Code 与 Codex 的四情况控制流、备份策略、断链写入**逐字相同**，
// 两处各写一份必然分叉（与 TASK-08 抽 `cleanup_crossbrain_orphans` 同一条理由）。
//
// 这里只保留 Claude **专有**的部分：目标文件名、`@~` 引用行、标记块内容构造。
//
// `SyncL0Result` 与三个标记常量以 `pub use` 转出，使既有路径
// `crossbrain_lib::adapters::claude_code::{SyncL0Result}` 继续可用
// （`examples/dryrun_claude.rs` 依赖它），调用方与测试无需改动。

pub use super::{SyncL0Result, MARKER_END, MARKER_NOTE, MARKER_START};

/// 备份后缀的本地别名：真实定义在共享层 [`super::MARKER_BACKUP_SUFFIX`]，
/// 保留短名是为避免本文件与测试的无意义全量改名。
use super::MARKER_BACKUP_SUFFIX as BACKUP_SUFFIX;

/// 标记块内的规则引用行。
///
/// `@~` 是 **Claude Code 专有**的路径展开语法，Spike A 已实测 Windows 下可用。
///
/// ⚠️ 正因它专属于 Claude，**不能**把这行照抄给 Codex——Codex 的标记块改为
/// 内联规则全文（见 `adapters/codex.rs` 文件头说明）。
const AGENTS_MD_REF: &str = "@~/.ai-profile/AGENTS.md";

/// 目标文件名。
const CLAUDE_MD_NAME: &str = "CLAUDE.md";

/// 技能文件名（三个 Adapter 通用，见 `ADAPTER_SPEC.md` 4.1）。
const SKILL_FILE_NAME: &str = "SKILL.md";

/// Claude Code 适配器。
///
/// # 关于 `base_dir`
///
/// 生产路径由 [`crate::paths::claude_dir`] 解析为 `~/.claude/`。
/// 但**测试绝不能在真实目录上跑**：这里执行的是真实写入，
/// 而真实 `~/.claude/CLAUDE.md` 还与另外 4 个工具硬链接共享同一 inode。
///
/// 因此提供 [`ClaudeCodeAdapter::with_base_dir`] 把根目录重定向到临时目录，
/// 让四情况协议与断链行为都能在完全隔离的沙箱里被验证。
#[derive(Debug, Clone, Default)]
pub struct ClaudeCodeAdapter {
    /// 配置根目录覆盖；`None` = 使用真实的 `~/.claude/`。
    base_dir: Option<PathBuf>,
}

impl ClaudeCodeAdapter {
    /// 生产用构造：指向真实的 `~/.claude/`。
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造一个把所有读写重定向到 `base_dir` 的实例。
    ///
    /// 两个用途：
    /// 1. **测试隔离**（主要目的）——真实文件与 4 个工具硬链接共享，不能被当作实验场
    /// 2. 未来支持 `ADAPTER_SPEC.md` 第 5 节 Level 3「用户手动指定路径」
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// 配置根目录。
    fn claude_dir_inner(&self) -> PathBuf {
        match &self.base_dir {
            Some(dir) => dir.clone(),
            None => paths::claude_dir(),
        }
    }

    /// `CLAUDE.md` 完整路径。
    fn claude_md_file(&self) -> PathBuf {
        self.claude_dir_inner().join(CLAUDE_MD_NAME)
    }

    /// 备份文件路径（`CLAUDE.md.crossbrain-backup`）。
    fn backup_file(&self) -> PathBuf {
        self.claude_dir_inner()
            .join(format!("{CLAUDE_MD_NAME}{BACKUP_SUFFIX}"))
    }

    /// `skills/` 目录。
    fn skills_dir(&self) -> PathBuf {
        self.claude_dir_inner().join("skills")
    }

    /// 指定技能的 `SKILL.md` 路径。
    fn skill_file(&self, slug_hash: &str) -> PathBuf {
        self.skills_dir().join(slug_hash).join(SKILL_FILE_NAME)
    }

    /// 生成标记块内容。
    ///
    /// 每次调用重新生成（而非从文件里读回旧块）：保证无论文件里原本是什么形态
    /// （被用户挪过位置、缩进过、混入了多余空行），最终都收敛到这个固定格式，
    /// 这也是情况二幂等的基础。
    fn build_marker_block() -> String {
        format!("{MARKER_START}\n{MARKER_NOTE}\n{AGENTS_MD_REF}\n{MARKER_END}")
    }

    /// 执行四情况协议，返回是否需要弹 UI 提示。
    ///
    /// 控制流本体在共享层 [`super::inject_marker_block`]——Claude Code 与 Codex
    /// 共用一份实现，本方法只负责提供 Claude 的三个差异项：
    /// 目标文件、备份文件、标记块内容。
    ///
    /// 四情况的判定顺序见共享层文档（命中即止）。
    ///
    /// # 关于 `rules_content`
    ///
    /// 本 Adapter **不使用**该参数：Claude 侧标记块的内容恒为
    /// `@~/.ai-profile/AGENTS.md` 一行引用（Spike A 已实测该引用在 Windows 下可展开），
    /// 规则本体由 `AGENTS.md` 承载，因此 `CLAUDE.md` 始终很小。
    ///
    /// 保留参数是给 `ADAPTER_SPEC.md` 3.5 的 Plan B 留入口：若未来 `@` 引用被证伪，
    /// 降级方案正是把 `rules_content` 直接内联进标记块，那时签名无需改动。
    /// 当前不实现未被验证的降级分支——避免造出「永远触发不到、也永远测不到」的死代码。
    ///
    /// ⚠️ Codex 的语义与此**相反**：它必须内联（Codex 不认识 `@` 语法）。
    /// 同一签名、两种语义是刻意的，不要为了「统一」而删掉这个参数。
    pub fn sync_l0_with_result(&self, rules_content: &str) -> Result<SyncL0Result, AdapterError> {
        let _ = rules_content;
        inject_marker_block(
            &self.claude_md_file(),
            &self.backup_file(),
            &Self::build_marker_block(),
        )
    }
}

// 断链写入与四情况协议均已完全迁至共享层（`super::write_breaking_hardlink`
// 与 `super::inject_marker_block`），本文件不再保留任何本地副本——
// D-03 约束的写入通道有且只有一个，副本就是分叉的开始。

impl Adapter for ClaudeCodeAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(self.claude_dir_inner().is_dir())
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
        // 也无需保护用户内容 → 直接写（与 Antigravity 侧一致）。
        fs::write(&file, content.as_bytes())
            .map_err(|e| AdapterError::WriteError(format!("{}：{e}", file.display())))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        // 清理规则与 Antigravity Adapter 完全一致（TASK-08 要求两者行为一致），
        // 故共用 `super::cleanup_crossbrain_orphans`；两边的差异只在 skills_dir 的取值。
        cleanup_crossbrain_orphans(&self.skills_dir(), active_slugs)
    }

    fn l0_backup(&self) -> Option<L0Backup> {
        // Claude Code 是**标记块注入型**：我们改的是用户的 `CLAUDE.md` 本身，
        // 首次注入前留了 `.crossbrain-backup`。因此「还原」在这里有明确含义
        // （TASK-19 / ADR-15）。
        //
        // 走 `self.*` 而非 `paths::*`：`with_base_dir` 注入的临时目录也必须能查询到
        // 自己的备份，否则测试只能去断言真实路径。
        Some(L0Backup {
            target_file: self.claude_md_file(),
            backup_file: self.backup_file(),
        })
    }

    fn uninstall(&self) -> Result<Vec<String>, AdapterError> {
        let mut actions = Vec::new();

        // ① 移除标记块（标记块外内容原样保留）。必须在删备份**之前**：
        //    备份是用户唯一的「回到最初」手段，移除成功后它才变成冗余。
        if let Some(desc) = remove_marker_block_from_file(&self.claude_md_file(), &self.backup_file())? {
            actions.push(desc);
        }

        // ② 删除备份文件（去痕）
        actions.extend(delete_backup_files(
            &self.backup_file(),
            &pre_restore_path(&self.claude_md_file()),
        ));

        // ③ 清空本工具 skills 目录下的全部 crossbrain-* 目录
        actions.extend(skill_cleanup_actions(self.cleanup_orphans(&[])?));

        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    //! 所有测试都在**临时目录**里跑（通过 `with_base_dir` 重定向），
    //! 绝不触碰真实的 `~/.claude/CLAUDE.md` ——那里有用户的规则，
    //! 且还与另外 4 个工具硬链接共享同一 inode。

    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 用「进程号 + 计数器」命名，不依赖时钟，且进程内互不冲突。
    static SEQ: AtomicU32 = AtomicU32::new(0);

    const USER_ORIGINAL: &str = "# 我的 Claude 规则\n\n- 用中文回答\n- 先给结论\n";

    /// 隔离的临时配置根目录，`Drop` 时自动清理。
    struct TempEnv {
        root: PathBuf,
    }

    impl TempEnv {
        fn new(tag: &str) -> Self {
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "crossbrain-cc-{}-{tag}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).expect("创建临时测试根目录失败");
            Self { root }
        }

        fn adapter(&self) -> ClaudeCodeAdapter {
            ClaudeCodeAdapter::with_base_dir(&self.root)
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.root.join(rel)
        }

        fn claude_md(&self) -> PathBuf {
            self.root.join(CLAUDE_MD_NAME)
        }

        fn backup(&self) -> PathBuf {
            self.root.join(format!("{CLAUDE_MD_NAME}{BACKUP_SUFFIX}"))
        }
    }

    impl Drop for TempEnv {
        fn drop(&mut self) {
            // Windows 上文件句柄可能延迟释放（本机实测过 os error 5）；
            // 清理失败不影响测试结论，忽略即可。
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// 断言两个路径**确实**共享同一 inode（硬链接前置条件）。
    ///
    /// # 为什么不用 `MetadataExt::file_index()`
    ///
    /// std 的 `file_index()` / `number_of_links()` 仍属 unstable
    /// （feature `windows_by_handle`），稳定通道编译不过。
    ///
    /// 改用**行为验证**，它其实比读 inode 更硬：直接证明了「就近覆写会互相污染」
    /// 这个我们真正要防的后果——向 `a` 就地写入后 `b` 内容跟着变，即同 inode。
    fn assert_shares_inode(a: &Path, b: &Path, original: &str) {
        let probe = "PROBE-共享 inode 验证\n";
        fs::write(a, probe).expect("探针写入失败");
        assert_eq!(
            fs::read_to_string(b).unwrap(),
            probe,
            "前置条件不成立：{} 与 {} 未共享同一 inode，本测试失去意义",
            a.display(),
            b.display()
        );
        // 复原：就地覆写同样作用于共享 inode，写回原文即可
        fs::write(a, original).expect("探针复原失败");
    }

    /// 断言两个路径**不再**共享同一 inode（断链已生效）。
    ///
    /// 同样用行为验证：断链后，就地覆写 `a` 不得再影响 `b`。
    fn assert_inode_detached(a: &Path, b: &Path) {
        let before = fs::read_to_string(b).expect("读取快照失败");
        fs::write(a, "PROBE-断链验证\n").expect("探针写入失败");
        assert_eq!(
            fs::read_to_string(b).unwrap(),
            before,
            "{} 仍被 {} 的就地覆写牵动 → 硬链接未断开",
            b.display(),
            a.display()
        );
    }

    /// 复现本机真实的 5 链接结构：一份内容、多个工具共享同一 inode。
    ///
    /// 返回所有路径，`[0]` 是原始路径，其余是兄弟工具路径。
    fn make_hardlinked_fixture(env: &TempEnv) -> Vec<PathBuf> {
        let paths = vec![
            env.path("user_profile.md"),           // ~/.ai-memory/user_profile.md
            env.claude_md(),                       // ~/.claude/CLAUDE.md
            env.path("cursor_user_profile.md"),    // ~/.cursor/rules/user_profile.md
            env.path("opencode_AGENTS.md"),        // ~/.config/opencode/AGENTS.md
            env.path("gemini_user_global.md"),     // ~/.gemini/config/rules/user_global.md
        ];
        fs::write(&paths[0], USER_ORIGINAL).expect("写入共享内容失败");
        for p in &paths[1..] {
            fs::hard_link(&paths[0], p).expect("创建硬链接失败（需 NTFS）");
        }
        paths
    }

    // ---------- detect ----------

    #[test]
    fn detect_reflects_claude_dir_existence() {
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
        let result = env.adapter().sync_l0_with_result("任意内容").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况一必须静默，不弹提示");

        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.starts_with(MARKER_START), "文件应以标记块开头：{content}");
        assert!(content.contains(MARKER_END), "缺少结束标志：{content}");
        assert!(
            content.contains(AGENTS_MD_REF),
            "缺少 @ 引用行：{content}"
        );
        assert_eq!(
            content,
            format!("{MARKER_START}\n{MARKER_NOTE}\n{AGENTS_MD_REF}\n{MARKER_END}"),
            "标记块格式必须逐字符固定"
        );
    }

    /// 情况一在同一目录不存在时也要能创建目录，且重复执行不重复追加。
    #[test]
    fn case1_then_case2_repeated_call_is_idempotent() {
        let env = TempEnv::new("case1-idem");
        let adapter = env.adapter();

        adapter.sync_l0_with_result("x").unwrap();
        let first = fs::read_to_string(env.claude_md()).unwrap();

        // 第二次进来已是情况二（文件已含标记块），结果必须完全相同
        let result = adapter.sync_l0_with_result("x").unwrap();
        assert_eq!(result, SyncL0Result::Silent);
        assert_eq!(
            fs::read_to_string(env.claude_md()).unwrap(),
            first,
            "重复同步不得累积标记块"
        );
    }

    // ---------- 情况二 ----------

    #[test]
    fn case2_replaces_marker_block_and_preserves_user_content() {
        let env = TempEnv::new("case2");
        fs::write(
            env.claude_md(),
            format!("{USER_ORIGINAL}\n\n<!-- CrossBrain:Start -->\n旧内容\n<!-- CrossBrain:End -->\n\n尾部用户补充\n"),
        )
        .unwrap();

        let result = env.adapter().sync_l0_with_result("new").unwrap();
        assert_eq!(result, SyncL0Result::Silent, "情况二必须静默");

        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.starts_with(USER_ORIGINAL), "块前的用户内容被改动：{content}");
        assert!(content.ends_with("尾部用户补充\n"), "块后的用户内容被改动：{content}");
        assert!(!content.contains("旧内容"), "旧标记块内容未被替换：{content}");
        assert!(content.contains(AGENTS_MD_REF), "新标记块未写入：{content}");
        assert_eq!(content.matches(MARKER_START).count(), 1, "标记块被重复插入：{content}");
    }

    /// 验收明确要求：情况二重复执行内容不变（幂等）。
    #[test]
    fn case2_is_idempotent_across_repeated_runs() {
        let env = TempEnv::new("case2-idem");
        let adapter = env.adapter();
        fs::write(env.claude_md(), format!("{USER_ORIGINAL}\n\n{0}", ClaudeCodeAdapter::build_marker_block())).unwrap();

        adapter.sync_l0_with_result("a").unwrap();
        let once = fs::read_to_string(env.claude_md()).unwrap();
        adapter.sync_l0_with_result("b").unwrap();
        let twice = fs::read_to_string(env.claude_md()).unwrap();

        assert_eq!(once, twice, "情况二重复执行必须字节级一致");
    }

    // ---------- 情况三 ----------

    #[test]
    fn case3_backs_up_then_appends_marker_block() {
        let env = TempEnv::new("case3");
        fs::write(env.claude_md(), USER_ORIGINAL).unwrap();

        let result = env.adapter().sync_l0_with_result("rules").unwrap();

        // ① 返回值必须携带备份路径（UI 靠它弹提示）
        assert_eq!(
            result,
            SyncL0Result::BackupCreated {
                backup_path: env.backup().to_string_lossy().to_string()
            }
        );

        // ② 备份内容 = 修改前的原文，一字不差
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            USER_ORIGINAL,
            "备份内容必须与原始 CLAUDE.md 完全一致"
        );

        // ③ 原文件：用户内容保留 + 标记块追加在末尾
        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.starts_with(USER_ORIGINAL), "用户内容被破坏：{content}");
        assert!(content.ends_with(MARKER_END), "标记块未追加到末尾：{content}");
        assert_eq!(
            content,
            format!("{}\n\n{}", USER_ORIGINAL.trim_end(), ClaudeCodeAdapter::build_marker_block()),
            "追加格式必须为「原文 + 空行 + 标记块」"
        );
    }

    /// 文件存在但为空时，不应产出以空行开头的文件。
    #[test]
    fn case3_handles_empty_existing_file() {
        let env = TempEnv::new("case3-empty");
        fs::write(env.claude_md(), "").unwrap();

        env.adapter().sync_l0_with_result("x").unwrap();

        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.starts_with(MARKER_START), "空文件被写了多余前导空行：{content:?}");
        assert!(env.backup().is_file(), "空文件同样应产生备份");
    }

    /// 备份必须是**独立文件**：若它也是硬链接，后续写入会污染这份原始快照。
    #[test]
    fn case3_backup_is_independent_not_a_hardlink() {
        let env = TempEnv::new("case3-backup-link");
        let fixture = make_hardlinked_fixture(&env);

        env.adapter().sync_l0_with_result("x").unwrap();

        let backup = env.backup();
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            USER_ORIGINAL,
            "备份内容不正确"
        );
        // 备份若是硬链接，后续写入会污染这份「原始快照」
        assert_inode_detached(&fixture[0], &backup);
    }

    // ---------- 情况四 ----------

    #[test]
    fn case4_skips_backup_when_backup_already_exists() {
        let env = TempEnv::new("case4");
        fs::write(env.claude_md(), USER_ORIGINAL).unwrap();
        fs::write(env.backup(), "OLD BACKUP\n").unwrap();

        let result = env.adapter().sync_l0_with_result("x").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况四不应重复弹提示");
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            "OLD BACKUP\n",
            "已有备份被覆盖——用户此前的干净快照丢失"
        );
        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.ends_with(MARKER_END), "标记块未追加：{content}");
        assert!(content.contains("用中文回答"), "用户内容丢失：{content}");
    }

    // ---------- 决策 D-03：断链写入（本任务最关键的安全约束）----------

    /// **核心安全测试**：`CLAUDE.md` 与其它 4 个工具硬链接共享同一 inode 时，
    /// `sync_l0` 后兄弟路径必须**一字未变**，且不再是同一个文件。
    #[test]
    fn sync_l0_breaks_hardlink_and_leaves_sibling_tools_untouched() {
        let env = TempEnv::new("hardlink");
        let fixture = make_hardlinked_fixture(&env);
        let claude_md = env.claude_md();

        // 前置条件：先证明它们**真的**共享 inode，否则本测试毫无意义（可能一直是假通过）
        assert_shares_inode(&fixture[0], &claude_md, USER_ORIGINAL);

        let result = env.adapter().sync_l0_with_result("rules").unwrap();
        assert_eq!(
            result,
            SyncL0Result::BackupCreated {
                backup_path: env.backup().to_string_lossy().to_string()
            }
        );

        // ① 其余工具路径的内容必须一字未变（这是 D-03 的全部意义）。
        //    注意 `fixture` 里含 CLAUDE.md 自身，要排除——它本就该被改。
        for (i, p) in fixture.iter().enumerate().filter(|(_, p)| **p != claude_md) {
            assert_eq!(
                fs::read_to_string(p).unwrap(),
                USER_ORIGINAL,
                "第 {i} 个兄弟路径被污染：{}",
                p.display()
            );
        }

        // ② CLAUDE.md 拿到了标记块 + 保留原内容
        let written = fs::read_to_string(&claude_md).unwrap();
        assert!(written.contains(MARKER_START), "标记块未写入：{written}");
        assert!(written.contains("用中文回答"), "原有内容未保留：{written}");

        // ③ 断链已生效。放最后：探针会改动 CLAUDE.md 自身的内容
        assert_inode_detached(&claude_md, &fixture[0]);
    }

    /// 情况二（替换已有标记块）同样必须断链——它是覆盖面最大的一条路径：
    /// 首次同步之后，此后每次同步都走这里。
    #[test]
    fn sync_l0_breaks_hardlink_in_case2_path_too() {
        let env = TempEnv::new("hardlink-case2");
        let fixture = make_hardlinked_fixture(&env);

        // 先做一次（情况三）——它已断链，这里再人为重建共享关系，
        // 专门验证「文件已有标记块」时的写入同样断开链接
        env.adapter().sync_l0_with_result("x").unwrap();
        let with_block = fs::read_to_string(env.claude_md()).unwrap();
        fs::write(&fixture[0], &with_block).unwrap();
        let _ = fs::remove_file(env.claude_md());
        fs::hard_link(&fixture[0], env.claude_md()).unwrap();

        assert_shares_inode(&fixture[0], &env.claude_md(), &with_block);

        // 情况二分支
        let result = env.adapter().sync_l0_with_result("x").unwrap();
        assert_eq!(result, SyncL0Result::Silent);

        assert_eq!(
            fs::read_to_string(&fixture[0]).unwrap(),
            with_block,
            "情况二路径污染了共享该 inode 的其它工具"
        );
        assert_inode_detached(&fixture[0], &env.claude_md());
    }

    /// 断链写入不得留下临时文件残留。
    #[test]
    fn write_leaves_no_temp_file_behind() {
        let env = TempEnv::new("no-temp");
        fs::write(env.claude_md(), USER_ORIGINAL).unwrap();

        env.adapter().sync_l0_with_result("x").unwrap();

        let leftovers: Vec<String> = fs::read_dir(&env.root)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains("tmp"))
            .collect();
        assert!(leftovers.is_empty(), "存在临时文件残留：{leftovers:?}");
    }

    // ---------- sync_l2 ----------

    #[test]
    fn sync_l2_writes_skill_md_with_frontmatter() {
        let env = TempEnv::new("l2");
        let slug = "crossbrain-vue3-perf-7c8e2a";
        let content = super::super::format_skill_md("vue3-perf", "# Vue3 性能优化\n\n正文");

        env.adapter().sync_l2(slug, &content).expect("写入应成功");

        let file = env.path(&format!("skills/{slug}/SKILL.md"));
        assert!(
            file.is_file(),
            "SKILL.md 未创建（skills/{{slug}}/ 应被自动创建）"
        );

        let written = fs::read_to_string(&file).unwrap();
        assert_eq!(written, content, "写入内容必须与传入完全一致");
        assert!(
            written.starts_with("---\nname: "),
            "必须以 YAML frontmatter 开头：{written}"
        );
    }

    /// 安全边界：绝不能写进 CrossBrain 命名空间之外（同 Antigravity 侧）。
    #[test]
    fn sync_l2_rejects_targets_outside_crossbrain_namespace() {
        let env = TempEnv::new("l2-guard");
        let adapter = env.adapter();

        for bad in [
            "grill-me",
            "design-taste-frontend",
            "crossbrain-../../evil",
            "crossbrain-a/b",
        ] {
            assert!(
                adapter.sync_l2(bad, "x").is_err(),
                "非法目标 {bad} 必须被拒绝"
            );
        }

        assert!(
            !env.path("skills").exists(),
            "被拒绝的写入不得留下任何痕迹"
        );
    }

    // ---------- cleanup_orphans ----------

    /// 验收核心：删孤儿、留活跃、绝不碰用户自建目录。
    #[test]
    fn cleanup_orphans_deletes_orphans_keeps_active_and_ignores_user_dirs() {
        let env = TempEnv::new("cleanup");
        let skills = env.path("skills");
        fs::create_dir_all(&skills).unwrap();

        for name in [
            "crossbrain-keep-111111",
            "crossbrain-orphan-222222",
            "grill-me",                // 用户自建
            "crossbrainx-impostor",    // 前缀相似，但不是 crossbrain-
        ] {
            let dir = skills.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(SKILL_FILE_NAME), "x").unwrap();
        }

        let active = vec!["crossbrain-keep-111111".to_string()];
        let report = env.adapter().cleanup_orphans(&active).unwrap();

        assert_eq!(
            report.deleted_dirs,
            vec!["crossbrain-orphan-222222".to_string()]
        );
        assert_eq!(report.kept_dirs, vec!["crossbrain-keep-111111".to_string()]);

        assert!(skills.join("crossbrain-keep-111111").is_dir(), "活跃目录被误删");
        assert!(!skills.join("crossbrain-orphan-222222").exists(), "孤儿目录未删除");
        assert!(skills.join("grill-me").is_dir(), "用户自建目录被误删");
        assert!(skills.join("crossbrainx-impostor").is_dir(), "前缀相似目录被误删");
    }

    #[test]
    fn cleanup_orphans_is_noop_when_skills_dir_missing() {
        let env = TempEnv::new("cleanup-noop");
        let report = env.adapter().cleanup_orphans(&[]).unwrap();
        assert_eq!(
            report,
            CleanupReport {
                deleted_dirs: vec![],
                kept_dirs: vec![],
            }
        );
    }

    #[test]
    fn cleanup_orphans_skips_plain_files_with_matching_prefix() {
        let env = TempEnv::new("cleanup-file");
        let skills = env.path("skills");
        fs::create_dir_all(&skills).unwrap();
        fs::write(skills.join("crossbrain-not-a-dir"), "x").unwrap();

        let report = env.adapter().cleanup_orphans(&[]).unwrap();

        assert!(
            report.deleted_dirs.is_empty(),
            "普通文件不应被当作孤儿目录删除"
        );
        assert!(
            skills.join("crossbrain-not-a-dir").is_file(),
            "普通文件被误删"
        );
    }

    // ---------- 契约一致性 ----------

    /// trait 的 `sync_l0` 只是丢掉分支结果，行为必须与 `sync_l0_with_result` 等价。
    #[test]
    fn trait_sync_l0_matches_with_result_path() {
        let env = TempEnv::new("trait-l0");
        fs::write(env.claude_md(), USER_ORIGINAL).unwrap();

        env.adapter().sync_l0("rules").expect("trait 方法应成功");

        // 情况三的副作用（备份 + 追加）必须照常发生
        assert!(env.backup().is_file(), "trait 路径未触发备份");
        let content = fs::read_to_string(env.claude_md()).unwrap();
        assert!(content.ends_with(MARKER_END), "trait 路径未追加标记块：{content}");
    }

    // 标记块协议本身的测试（判定与替换的一致性等）已随实现迁至共享层
    // `adapters/mod.rs` 的测试模块——协议在哪里，验证就在哪里。

    /// 卸载全周期：同步注入 → 卸载 → 用户内容原样回来、备份与技能目录清空（TASK-14）。
    #[test]
    fn uninstall_restores_user_content_and_removes_traces() {
        let env = TempEnv::new("uninstall");
        let adapter = env.adapter();

        // 用户原有内容 + 一个自建技能目录（绝不能被误删）
        fs::write(env.claude_md(), USER_ORIGINAL).unwrap();
        let skill = env.path("skills").join("crossbrain-demo-abc123");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "---\nname: demo\n---\n").unwrap();
        let user_skill = env.path("skills").join("my-own-skill");
        fs::create_dir_all(&user_skill).unwrap();

        adapter.sync_l0("规则正文").unwrap();
        assert!(env.backup().is_file());

        let actions = adapter.uninstall().unwrap();
        assert!(
            actions.iter().any(|a| a.contains("原样保留")),
            "动作描述要向用户说明内容未丢：{actions:?}"
        );

        let restored = fs::read_to_string(env.claude_md()).unwrap();
        assert_eq!(
            restored, USER_ORIGINAL,
            "标记块移除后必须回到用户原文（尾部空白归整为单个换行）"
        );
        assert!(!env.backup().exists(), "备份必须被删除（去痕）");
        assert!(!skill.exists(), "crossbrain-* 技能目录必须被删除");
        assert!(user_skill.exists(), "用户自建技能绝不能被误删");
    }
}
