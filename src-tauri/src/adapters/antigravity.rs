//! antigravity.rs — Antigravity IDE 的 Adapter 实现。
//!
//! # 目标路径（依据 `docs/tech/ADAPTER_SPEC.md` 第 2 节 + Spike D 实测结论）
//!
//! | 用途 | 路径 |
//! |:---|:---|
//! | 安装检测 | `~/.gemini/config/` |
//! | L0 全局规则 | `~/.gemini/config/rules/crossbrain-L0.md`（**直接全量覆盖**） |
//! | L2 技能知识 | `~/.gemini/config/skills/{slug-hash}/SKILL.md`（原生懒加载） |
//! | 孤儿清理 | `~/.gemini/config/skills/` 下的 `crossbrain-*` 目录 |
//!
//! # ⚠️ 核心约束：绝不触碰用户的既有规则文件
//!
//! Spike D 实测结论：`~/.gemini/config/rules/` 下**所有** `.md` 都会被全局读取，
//! 且用户已在该目录放了自己的 `user_global.md`。
//!
//! 因此本 Adapter 采用**独立文件**策略——只写 `crossbrain-L0.md`，
//! 绝不读写 `user_global.md` 或任何非 `crossbrain-` 前缀的文件。
//!
//! > **实测补充（2026-09-14）**：该 `user_global.md` 通过**硬链接**与
//! > `~/.claude/CLAUDE.md`、`~/.cursor/rules/user_profile.md`、
//! > `~/.config/opencode/AGENTS.md`、`~/.ai-memory/user_profile.md`
//! > 共享同一个 inode（`fsutil hardlink list` 已验证）。
//! > 对其中任意一个做「就地覆写」都会同时改变全部 5 个工具看到的规则。
//! > 这是必须使用独立文件的**又一个**硬性理由。
//!
//! # 铁律
//!
//! - **ADR-09**：孤儿清理只允许删除 `crossbrain-` 前缀的目录
//! - **L-02**：路径一律取自 [`crate::paths`]，不得自行拼接或硬编码盘符

use std::fs;
use std::path::PathBuf;

use super::{
    cleanup_crossbrain_orphans, ensure_crossbrain_slug, Adapter, AdapterError, CleanupReport,
};
use crate::paths;

/// L0 规则文件名（固定值，见 `ADAPTER_SPEC.md` 2.2）。
const L0_FILE_NAME: &str = "crossbrain-L0.md";

/// 技能文件名（固定值，各 Adapter 通用，见 `ADAPTER_SPEC.md` 4.1）。
const SKILL_FILE_NAME: &str = "SKILL.md";

/// Antigravity IDE 适配器。
///
/// # 关于 `base_dir`
///
/// 生产路径由 [`crate::paths::gemini_config_dir`] 解析为 `~/.gemini/config/`。
/// 但**测试绝不能在真实目录上跑**：`cleanup_orphans()` 执行的是真实删除，
/// 而用户真实的 `~/.gemini/config/skills/` 下既有自建技能，也有历史遗留目录。
///
/// 因此提供 [`AntigravityAdapter::with_base_dir`] 把根目录重定向到临时目录，
/// 让删除行为能在完全隔离的沙箱里被验证。
#[derive(Debug, Clone, Default)]
pub struct AntigravityAdapter {
    /// 配置根目录覆盖；`None` = 使用真实的 `~/.gemini/config/`。
    base_dir: Option<PathBuf>,
}

impl AntigravityAdapter {
    /// 生产用构造：指向真实的 `~/.gemini/config/`。
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造一个把所有读写重定向到 `base_dir` 的实例。
    ///
    /// 两个用途：
    /// 1. **测试隔离**（主要目的）——真实目录下有用户数据，不能被当作实验场
    /// 2. 未来支持 `ADAPTER_SPEC.md` 第 5 节 Level 3「用户手动指定路径」
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// 配置根目录。
    fn config_dir(&self) -> PathBuf {
        match &self.base_dir {
            Some(dir) => dir.clone(),
            None => paths::gemini_config_dir(),
        }
    }

    /// `rules/` 目录。
    fn rules_dir(&self) -> PathBuf {
        self.config_dir().join("rules")
    }

    /// L0 规则文件路径（`rules/crossbrain-L0.md`）。
    fn l0_file(&self) -> PathBuf {
        self.rules_dir().join(L0_FILE_NAME)
    }

    /// `skills/` 目录。
    fn skills_dir(&self) -> PathBuf {
        self.config_dir().join("skills")
    }

    /// 指定技能的 `SKILL.md` 路径。
    fn skill_file(&self, slug_hash: &str) -> PathBuf {
        self.skills_dir().join(slug_hash).join(SKILL_FILE_NAME)
    }
}

impl Adapter for AntigravityAdapter {
    fn detect(&self) -> Result<bool, AdapterError> {
        Ok(self.config_dir().is_dir())
    }

    fn sync_l0(&self, rules_content: &str) -> Result<(), AdapterError> {
        let dir = self.rules_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| AdapterError::DirectoryCreateFailed(format!("{}：{e}", dir.display())))?;

        // 直接全量覆盖：该文件 100% 由 CrossBrain 托管，不含用户内容（ADAPTER_SPEC 2.2）。
        // 注意这里**只**写 crossbrain-L0.md——同目录下的 user_global.md 不受任何影响。
        let file = self.l0_file();
        fs::write(&file, rules_content.as_bytes())
            .map_err(|e| AdapterError::WriteError(format!("{}：{e}", file.display())))
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

        fs::write(&file, content.as_bytes())
            .map_err(|e| AdapterError::WriteError(format!("{}：{e}", file.display())))
    }

    fn cleanup_orphans(&self, active_slugs: &[String]) -> Result<CleanupReport, AdapterError> {
        // 清理规则与 Claude Code Adapter 完全一致（TASK-08 要求两者行为一致），
        // 故共用 `super::cleanup_crossbrain_orphans`；两边的差异只在 skills_dir 的取值。
        cleanup_crossbrain_orphans(&self.skills_dir(), active_slugs)
    }
}

#[cfg(test)]
mod tests {
    //! 所有测试都在**临时目录**里跑（通过 `with_base_dir` 重定向），
    //! 绝不触碰真实的 `~/.gemini/config/` ——那里有用户的规则和自建技能。

    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 用「进程号 + 计数器」而非时间戳命名：不依赖时钟，且计数器保证
    /// 同一进程内多个测试拿到互不冲突的目录。
    static SEQ: AtomicU32 = AtomicU32::new(0);

    /// 隔离的临时配置根目录，`Drop` 时自动清理。
    struct TempEnv {
        root: PathBuf,
    }

    impl TempEnv {
        fn new(tag: &str) -> Self {
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!(
                "crossbrain-ag-{}-{tag}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).expect("创建临时测试根目录失败");
            Self { root }
        }

        fn adapter(&self) -> AntigravityAdapter {
            AntigravityAdapter::with_base_dir(&self.root)
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.root.join(rel)
        }
    }

    impl Drop for TempEnv {
        fn drop(&mut self) {
            // Windows 上刚写入的文件句柄可能延迟释放（本机实测过 os error 5），
            // 清理失败不影响测试结论，忽略即可。
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// 建一个带 SKILL.md 的技能目录。
    fn make_skill(skills_dir: &Path, name: &str) {
        let dir = skills_dir.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(SKILL_FILE_NAME), "placeholder").unwrap();
    }

    // ---------- detect ----------

    #[test]
    fn detect_reflects_config_dir_existence() {
        let env = TempEnv::new("detect");
        let adapter = env.adapter();

        assert!(adapter.detect().unwrap(), "配置目录存在时 → true");

        fs::remove_dir_all(&env.root).unwrap();
        assert!(!adapter.detect().unwrap(), "配置目录不存在时 → false");
    }

    // ---------- sync_l0 ----------

    #[test]
    fn sync_l0_writes_exact_content_and_is_idempotent() {
        let env = TempEnv::new("l0");
        let adapter = env.adapter();
        let content = "# 全局规则\n\n- 交付导向\n- 简体中文\n";

        adapter.sync_l0(content).expect("首次写入应成功");

        let file = env.path("rules/crossbrain-L0.md");
        assert!(
            file.is_file(),
            "rules/crossbrain-L0.md 未创建（rules/ 应被自动创建）"
        );
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            content,
            "文件内容必须与传入的 rules_content 完全一致"
        );

        let snapshot = fs::read(&file).unwrap();
        adapter.sync_l0(content).expect("重复写入必须成功（幂等）");
        assert_eq!(
            fs::read(&file).unwrap(),
            snapshot,
            "重复写入不得改变文件内容"
        );
    }

    /// 铁律 L-01 的 Antigravity 侧对应约束：`sync_l0` 只准写 `crossbrain-L0.md`，
    /// 同目录下的用户既有规则必须一字不动。
    #[test]
    fn sync_l0_never_touches_existing_user_rules() {
        let env = TempEnv::new("l0-isolation");
        let rules = env.path("rules");
        fs::create_dir_all(&rules).unwrap();

        // 真实机器上该文件确实存在（且还硬链接到 ~/.claude/CLAUDE.md 等 4 处）
        let user_file = rules.join("user_global.md");
        fs::write(&user_file, "USER ORIGINAL RULES\n").unwrap();

        env.adapter().sync_l0("# CrossBrain 规则").unwrap();

        assert_eq!(
            fs::read_to_string(&user_file).unwrap(),
            "USER ORIGINAL RULES\n",
            "用户既有规则文件被改动了！"
        );
        assert!(
            env.path("rules/crossbrain-L0.md").is_file(),
            "独立文件未生成，可能与用户文件混淆"
        );
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

    /// 安全边界：绝不能写进 CrossBrain 命名空间之外。
    /// 真实机器上 `skills/` 里就有用户自建的 `grill-me`、`design-taste-frontend`。
    #[test]
    fn sync_l2_rejects_targets_outside_crossbrain_namespace() {
        let env = TempEnv::new("l2-guard");
        let adapter = env.adapter();

        for bad in [
            "grill-me",                    // 用户自建技能
            "design-taste-frontend",       // 用户自建技能
            "crossbrain-../../evil",       // 目录穿越
            "crossbrain-a/b",              // 多级路径
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

        make_skill(&skills, "crossbrain-keep-111111");
        make_skill(&skills, "crossbrain-orphan-222222");
        make_skill(&skills, "grill-me"); // 用户自建
        make_skill(&skills, "crossbrainx-impostor"); // 前缀相似，但不是 crossbrain-

        let active = vec!["crossbrain-keep-111111".to_string()];
        let report = env.adapter().cleanup_orphans(&active).unwrap();

        assert_eq!(
            report.deleted_dirs,
            vec!["crossbrain-orphan-222222".to_string()]
        );
        assert_eq!(report.kept_dirs, vec!["crossbrain-keep-111111".to_string()]);

        assert!(
            skills.join("crossbrain-keep-111111").is_dir(),
            "活跃目录被误删"
        );
        assert!(
            !skills.join("crossbrain-orphan-222222").exists(),
            "孤儿目录未删除"
        );
        assert!(skills.join("grill-me").is_dir(), "用户自建目录被误删");
        assert!(
            skills.join("crossbrainx-impostor").is_dir(),
            "前缀相似目录被误删"
        );
    }

    /// `skills/` 不存在时应静默返回空报告，而不是报错。
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

    /// `skills/` 下若有名为 `crossbrain-*` 的**普通文件**，应跳过而非报错或删除。
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

    /// 全部技能都活跃时不得删除任何目录（幂等的另一面）。
    #[test]
    fn cleanup_orphans_keeps_all_when_all_active() {
        let env = TempEnv::new("cleanup-all-active");
        let skills = env.path("skills");
        fs::create_dir_all(&skills).unwrap();
        make_skill(&skills, "crossbrain-a-111111");
        make_skill(&skills, "crossbrain-b-222222");

        let active = vec![
            "crossbrain-a-111111".to_string(),
            "crossbrain-b-222222".to_string(),
        ];
        let report = env.adapter().cleanup_orphans(&active).unwrap();

        assert!(
            report.deleted_dirs.is_empty(),
            "没有孤儿时不应删除任何东西"
        );
        assert_eq!(report.kept_dirs.len(), 2);
    }
}
