//! opencode.rs — OpenCode（SST）的 Adapter 实现。
//!
//! # 落点探测（2026-09-15 二进制取证 + 无头探针实证，详见 `SPIKE_RESULTS.md`）
//!
//! | 用途 | 路径 | 证据 |
//! |:---|:---|:---|
//! | 安装检测 | `~/.config/opencode/` | 本机实存（v1.18.30） |
//! | L0 全局规则 | `~/.config/opencode/AGENTS.md` | 二进制运行时代码：全局指令数组 = `[join(config,"AGENTS.md"), join(home,".claude","CLAUDE.md")]` |
//! | L2 技能知识 | `~/.config/opencode/skills/{slug}/SKILL.md` | 探针写入后 `opencode debug skill` 实证列出（中立目录执行） |
//! | 孤儿清理 | `~/.config/opencode/skills/` 下的 `crossbrain-*` 目录 | 同上 |
//! | 备份 | `~/.config/opencode/AGENTS.md.crossbrain-backup` | 共享层四情况协议 |
//!
//! # L0 形态：单文件 + 内联全文（与 Codex 同形态，**不可照搬** Antigravity/Claude）
//!
//! - OpenCode 只读 `AGENTS.md` 这一个文件 → 写独立文件（Antigravity 形态）= 永不被读；
//! - OpenCode 不认识 `@` 引用语法（那是 Claude Code 专有）→ 必须内联规则全文。
//!   旁证：OpenCode 会**原样加载** `~/.claude/CLAUDE.md`（`disableClaudeCodePrompt`
//!   关闭才跳过），Claude 标记块里的 `@~/.ai-profile/AGENTS.md` 对它只是一行普通文本。
//!
//! # ⚠️ 本落点是「全局记忆中心」硬链接组成员
//!
//! 真机上 `~/.config/opencode/AGENTS.md` 与 `~/.ai-memory/user_profile.md`、
//! `~/.cursor/rules/user_profile.md`、`~/.gemini/config/rules/user_global.md`
//! 是**同一 inode（links=4）**——用户手工维护的全局记忆分发。
//! 首次注入标记块时，共享层的 [`super::write_breaking_hardlink`] 会断开本文件的
//! 硬链接（其余 3 个链接内容不受影响），且首次改动前有 `.crossbrain-backup` +
//! 前端 ADR-15 断链告知闸门兜底。**这正是 D-03 决策存在的理由。**
//!
//! # 与 Codex 的差异
//!
//! - **无遮蔽文件**：二进制取证未见 `AGENTS.override.md` 式的高优先级覆盖机制
//!   （全局指令恒取 `config/AGENTS.md` + 项目层 `AGENTS.md` 去重合并），
//!   故本 Adapter 不做 Codex 那样的 override 拦截；
//!   用户在 `opencode.jsonc` 里显式配置 `instructions` 属用户自主行为，不属遮蔽。
//! - **多技能根**：OpenCode 还会扫描 `~/.agents/skills/`（86 个第三方技能）与
//!   `~/.claude/skills/`（Claude 兼容）。本 Adapter **只写自己的根**
//!   `~/.config/opencode/skills/`，其余零引用（写入边界铁律）。
//!   同名技能在 OpenCode 模型视图中的去重是宿主自身行为，不由我们干预。
//!
//! # ⛔ 严禁触碰的路径
//!
//! | 路径 | 是什么 |
//! |:---|:---|
//! | `~/.config/opencode/opencode.jsonc` | OpenCode 主配置 |
//! | `~/.config/opencode/node_modules/`、`bun.lock`、`package.json`、`.gitignore` | 插件运行时数据 |
//!
//! # 铁律
//!
//! - **L-01**：标记块格式逐字符固定，且永不整体覆盖用户文件
//! - **ADR-09**：孤儿清理只允许删除 `crossbrain-` 前缀的目录
//! - **L-02**：路径一律取自 [`crate::paths`]

use std::fs;
use std::path::PathBuf;

use super::{
    cleanup_crossbrain_orphans, delete_backup_files, ensure_crossbrain_slug,
    ensure_no_marker_in_content, inject_marker_block, pre_restore_path,
    remove_marker_block_from_file, skill_cleanup_actions, Adapter, AdapterError, CleanupReport,
    L0Backup, MARKER_END, MARKER_NOTE, MARKER_START,
};
use crate::paths;

/// 转出共享结果类型，使 `adapters::opencode::SyncL0Result` 与其它 Adapter 路径对称。
pub use super::SyncL0Result;

/// 备份后缀的本地别名（真实定义在共享层 [`super::MARKER_BACKUP_SUFFIX`]）。
use super::MARKER_BACKUP_SUFFIX as BACKUP_SUFFIX;

/// 目标文件名。
const AGENTS_MD_NAME: &str = "AGENTS.md";

/// 技能文件名（三个 Adapter 通用，见 `ADAPTER_SPEC.md` 4.1）。
const SKILL_FILE_NAME: &str = "SKILL.md";

/// 技能根目录名（OpenCode 的 `{skill,skills}` 两个 glob 都认，取与 Codex 一致的复数）。
const SKILLS_DIR_NAME: &str = "skills";

/// OpenCode 适配器。
///
/// # 关于 `base_dir`
///
/// 生产路径由 [`crate::paths::opencode_dir`] 解析为 `~/.config/opencode/`。
/// 测试必须走 [`OpenCodeAdapter::with_base_dir`] 重定向到临时目录——
/// 真实 `AGENTS.md` 是硬链接组成员（用户的全局记忆中心），绝不能在测试里碰。
#[derive(Debug, Clone, Default)]
pub struct OpenCodeAdapter {
    /// 配置根目录覆盖；`None` = 使用真实的 `~/.config/opencode/`。
    base_dir: Option<PathBuf>,
}

impl OpenCodeAdapter {
    /// 生产用构造：指向真实的 `~/.config/opencode/`。
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
    fn opencode_dir_inner(&self) -> PathBuf {
        match &self.base_dir {
            Some(dir) => dir.clone(),
            None => paths::opencode_dir(),
        }
    }

    /// `AGENTS.md` 完整路径。
    fn agents_md_file(&self) -> PathBuf {
        self.opencode_dir_inner().join(AGENTS_MD_NAME)
    }

    /// 备份文件路径（`AGENTS.md.crossbrain-backup`）。
    fn backup_file(&self) -> PathBuf {
        self.opencode_dir_inner()
            .join(format!("{AGENTS_MD_NAME}{BACKUP_SUFFIX}"))
    }

    /// `skills/` 目录。
    fn skills_dir(&self) -> PathBuf {
        self.opencode_dir_inner().join(SKILLS_DIR_NAME)
    }

    /// 指定技能的 `SKILL.md` 路径。
    fn skill_file(&self, slug_hash: &str) -> PathBuf {
        self.skills_dir().join(slug_hash).join(SKILL_FILE_NAME)
    }

    /// 构造 OpenCode 的标记块：**内联规则全文**（同 Codex，理由见模块文档）。
    ///
    /// 返回 `Result` 的原因与 Codex 相同：内联内容含 `MARKER_END` 时
    /// 非贪婪正则会提前闭合，必须写入前拒绝（ADR-14「失败可见」）。
    fn build_marker_block(rules_content: &str) -> Result<String, AdapterError> {
        ensure_no_marker_in_content(rules_content)?;

        let body = rules_content.trim_end();
        Ok(if body.is_empty() {
            format!("{MARKER_START}\n{MARKER_NOTE}\n{MARKER_END}")
        } else {
            format!("{MARKER_START}\n{MARKER_NOTE}\n{body}\n{MARKER_END}")
        })
    }

    /// 执行四情况协议，返回是否需要弹 UI 提示。
    ///
    /// 控制流本体在共享层 [`super::inject_marker_block`]，与 Codex Adapter
    /// 完全共用——差异只有目标文件、备份文件、标记块内容三项。
    pub fn sync_l0_with_result(&self, rules_content: &str) -> Result<SyncL0Result, AdapterError> {
        let marker_block = Self::build_marker_block(rules_content)?;
        inject_marker_block(&self.agents_md_file(), &self.backup_file(), &marker_block)
    }
}

impl Adapter for OpenCodeAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(self.opencode_dir_inner().is_dir())
    }

    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> {
        self.sync_l0_with_result(rules_content).map(|_| ())
    }

    fn sync_l2(&self, slug_hash: &str, content: &str) -> Result<(), AdapterError> {
        ensure_crossbrain_slug(slug_hash)?;

        let file = self.skill_file(slug_hash);
        let dir = file
            .parent()
            .ok_or_else(|| AdapterError::Other(format!("非法路径：{}", file.display())))?;
        fs::create_dir_all(dir)
            .map_err(|e| AdapterError::DirectoryCreateFailed(format!("{}：{e}", dir.display())))?;

        fs::write(&file, content.as_bytes())
            .map_err(|e| AdapterError::WriteError(format!("{}：{e}", file.display())))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        cleanup_crossbrain_orphans(&self.skills_dir(), active_slugs)
    }

    fn skills_root(&self) -> PathBuf {
        self.skills_dir()
    }

    fn l0_backup(&self) -> Option<L0Backup> {
        // 标记块注入型：改的是用户的 `AGENTS.md` 本身，首次注入前留 `.crossbrain-backup`。
        // OpenCode 的 AGENTS.md 还是硬链接组成员——备份是断链前的原始件，尤其珍贵。
        Some(L0Backup {
            target_file: self.agents_md_file(),
            backup_file: self.backup_file(),
        })
    }

    fn uninstall(&self) -> Result<Vec<String>, AdapterError> {
        let mut actions = Vec::new();

        // ① 移除标记块（内联的规则全文随之消失，用户自己的指令原样保留）。
        //    顺序是安全属性：先移块、后删备份。
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

        // ③ 清空 skills 目录下的 crossbrain-* 目录
        actions.extend(skill_cleanup_actions(self.cleanup_orphans(&[])?));

        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    //! 所有测试都在**临时目录**里跑（通过 `with_base_dir` 重定向），
    //! 绝不触碰真实的 `~/.config/opencode/AGENTS.md`——
    //! 那里是用户「全局记忆中心」硬链接组的成员（links=4）。

    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 用「进程号 + 计数器」命名，不依赖时钟，且进程内互不冲突。
    static SEQ: AtomicU32 = AtomicU32::new(0);

    const USER_ORIGINAL: &str = "# 我的 OpenCode 全局指令\n\n- 用中文回答\n";

    /// 隔离的临时配置根目录，`Drop` 时自动清理。
    struct TempEnv {
        root: PathBuf,
    }

    impl TempEnv {
        fn new(tag: &str) -> Self {
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "crossbrain-opencode-{}-{tag}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).expect("创建临时测试根目录失败");
            Self { root }
        }

        fn adapter(&self) -> OpenCodeAdapter {
            OpenCodeAdapter::with_base_dir(&self.root)
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
    fn detect_reflects_opencode_dir_existence() {
        let env = TempEnv::new("detect");
        let adapter = env.adapter();

        assert!(adapter.detect().unwrap(), "配置目录存在时 → true");

        fs::remove_dir_all(&env.root).unwrap();
        assert!(!adapter.detect().unwrap(), "配置目录不存在时 → false");
    }

    // ---------- 情况一（含内联断言） ----------

    #[test]
    fn case1_creates_file_with_marker_block_when_missing() {
        let env = TempEnv::new("case1");
        let result = env.adapter().sync_l0_with_result("用中文回答").unwrap();

        assert_eq!(result, SyncL0Result::Silent, "情况一必须静默，不弹提示");

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert_eq!(
            content,
            format!("{MARKER_START}\n{MARKER_NOTE}\n用中文回答\n{MARKER_END}"),
            "标记块格式必须逐字符固定"
        );
    }

    /// 内联是 OpenCode 与 Claude 的**核心差异**：规则全文必须在块内。
    #[test]
    fn case1_inlines_full_rules_content_not_a_reference() {
        let env = TempEnv::new("inline");
        let rules = "# 全局规则\n\n- 先给结论\n- 不确定就说不确定";
        env.adapter().sync_l0_with_result(rules).unwrap();

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.contains(rules), "规则全文未被内联：\n{content}");
        assert!(
            !content.contains("@~/.ai-profile/AGENTS.md"),
            "出现了 Claude 专有的 @ 引用语法——OpenCode 不认识它，会导致同步静默失效"
        );
    }

    #[test]
    fn case1_then_case2_repeated_call_is_idempotent() {
        let env = TempEnv::new("case1-idem");
        let adapter = env.adapter();

        adapter.sync_l0_with_result("规则内容").unwrap();
        let first = fs::read_to_string(env.agents_md()).unwrap();

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
        let adapter = OpenCodeAdapter::with_base_dir(&nested);

        adapter.sync_l0_with_result("规则").unwrap();
        assert!(
            nested.join(AGENTS_MD_NAME).is_file(),
            "父目录不存在时应自动创建"
        );
    }

    // ---------- 情况二 / 三 / 四 ----------

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

    #[test]
    fn case2_detection_rejects_unclosed_marker() {
        let env = TempEnv::new("unclosed");
        let broken = format!("{USER_ORIGINAL}\n{MARKER_START}\n没有结束标志\n");
        fs::write(env.agents_md(), &broken).unwrap();

        let result = env.adapter().sync_l0_with_result("规则").unwrap();

        assert!(
            matches!(result, SyncL0Result::BackupCreated { .. }),
            "未闭合的标记块被误判为情况二，实际：{result:?}"
        );
        assert!(env.backup().is_file(), "应已创建备份");
    }

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
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            USER_ORIGINAL,
            "备份内容与原始内容不一致"
        );

        let content = fs::read_to_string(env.agents_md()).unwrap();
        assert!(content.contains("用中文回答"), "原有内容未保留：{content}");
        assert!(content.contains(MARKER_START), "标记块未追加：{content}");
    }

    #[test]
    fn case4_does_not_overwrite_existing_backup() {
        let env = TempEnv::new("case4");
        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
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

    // ---------- 硬链接隔离（本落点的真实场景：links=4 的记忆中心成员） ----------

    /// 真机上 `~/.config/opencode/AGENTS.md` 与其它 3 个文件同 inode。
    /// 首次注入必须**断链写入**：兄弟路径内容一个字节都不能变。
    #[test]
    fn sync_l0_breaks_hardlink_without_polluting_siblings() {
        let env = TempEnv::new("hardlink");
        let sibling = env.path("ai-memory_user_profile.md");

        fs::write(env.agents_md(), USER_ORIGINAL).unwrap();
        fs::hard_link(env.agents_md(), &sibling).expect("创建硬链接失败（需 NTFS）");

        let result = env.adapter().sync_l0_with_result("规则").unwrap();
        assert!(
            matches!(result, SyncL0Result::BackupCreated { .. }),
            "硬链接成员首注必须走情况三（备份+追加），实际：{result:?}"
        );

        assert_eq!(
            fs::read_to_string(&sibling).unwrap(),
            USER_ORIGINAL,
            "兄弟路径被污染——说明写入没有断开硬链接，用户的记忆中心被改了"
        );
        assert!(fs::read_to_string(env.agents_md())
            .unwrap()
            .contains(MARKER_START));
        assert_eq!(
            fs::read_to_string(env.backup()).unwrap(),
            USER_ORIGINAL,
            "断链前的原始件必须进备份"
        );
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

        adapter.sync_l2(slug, body).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), body, "幂等：重复写入不变");
    }

    #[test]
    fn sync_l2_rejects_slug_outside_namespace() {
        let env = TempEnv::new("l2-reject");
        let adapter = env.adapter();

        assert!(adapter.sync_l2("grill-me", "x").is_err(), "未拦截用户技能名");
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

        // 模拟 OpenCode skills 目录的两类共存内容
        fs::create_dir_all(skills.join("user-own-skill")).unwrap(); // 用户自建
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
        assert!(skills.join("user-own-skill").is_dir(), "用户自建技能被删了");
        assert!(!skills.join("crossbrain-old-000000").exists());
        assert!(skills.join("crossbrain-keep-111111").is_dir());
    }

    #[test]
    fn cleanup_is_noop_when_skills_dir_missing() {
        let env = TempEnv::new("cleanup-missing");
        let report = env.adapter().cleanup_orphans(&[]).unwrap();
        assert!(report.deleted_dirs.is_empty());
        assert!(report.kept_dirs.is_empty());
    }

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
    }
}
