//! adapter_consistency.rs — TASK-08 验收：各 Adapter 的孤儿清理行为必须**逐项一致**。
//!
//! # 为什么单独建一个集成测试文件
//!
//! TASK-05 / TASK-06 各自的 `cleanup_orphans()` 已有单元测试，但它们跑在
//! **各自的测试夹具**里，断言的是「我这份实现符合预期」。TASK-08 验收要求的是
//! 「**彼此行为一致**」——这是另一个命题，只有把多个 Adapter 放进同一组用例、
//! 对同一组输入断言**完全相同的输出**才能证明。
//!
//! 放在 `tests/` 目录（集成测试）还有一个附带好处：它只能看到 `pub` API，
//! 因此同时验证了「外部调用方拿着 `Box<dyn Adapter>` 调用时行为不会分叉」——
//! 这正是 TASK-13 同步流程真实的调用形态。
//!
//! 清理逻辑本身共用 [`crossbrain_lib::adapters::cleanup_crossbrain_orphans`]，
//! 共用的意义就在这里：行为一致不是「碰巧对齐」，而是结构上不可能分叉。
//! 这些用例的作用是**锁死**这个性质，防止日后有人给某个 Adapter 加私货分支。
//!
//! # 参与对比的 Adapter
//!
//! Antigravity IDE / Claude Code / Codex（Codex 于 TASK-18 增补）。
//! 新增 Adapter 时**必须**加进本文件的对比列表——只加单元测试不算数，
//! 「我这份对」推不出「彼此一致」。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use crossbrain_lib::adapters::antigravity::AntigravityAdapter;
use crossbrain_lib::adapters::claude_code::ClaudeCodeAdapter;
use crossbrain_lib::adapters::codex::CodexAdapter;
use crossbrain_lib::adapters::{Adapter, CleanupReport};

// ============================================================================
// 临时目录夹具
// ============================================================================

/// 自清理的临时根目录。
///
/// `Drop` 里忽略删除失败：Windows 上刚被程序关闭的句柄释放有延迟，
/// 目录可能短暂无法删除——进程退出后残留也无害（位于系统临时目录）。
struct TempRoot(PathBuf);

impl TempRoot {
    fn new(tag: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "crossbrain-t8-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("创建临时根目录失败");
        TempRoot(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// ============================================================================
// 用例工具
// ============================================================================

/// 在 `root` 下按描述建出目录树。
///
/// 每条描述形如 `"d:crossbrain-x-111111"`（目录，内含一个 `SKILL.md`）
/// 或 `"f:crossbrain-x-111111"`（同名前缀的普通文件）。
/// 目录里放一个文件是刻意的——空目录用 `remove_dir_all` 删不掉东西，
/// 只有非空目录才真正检验「删的是目录树」。
fn make_tree(root: &Path, entries: &[&str]) {
    fs::create_dir_all(root).expect("创建 skills 目录失败");
    for spec in entries {
        let (is_dir, name) = match spec.split_once(':') {
            Some(("d", n)) => (true, n),
            Some(("f", n)) => (false, n),
            _ => panic!("条目描述必须以 d: 或 f: 开头：{spec}"),
        };
        let target = root.join(name);
        if is_dir {
            fs::create_dir_all(&target).expect("创建子目录失败");
            fs::write(target.join("SKILL.md"), "占位内容").expect("写入 SKILL.md 失败");
        } else {
            fs::write(&target, "占位内容").expect("写入普通文件失败");
        }
    }
}

/// 列出目录下的条目名（排序后返回），用于比对清理结果。
fn list_sorted(dir: &Path) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("读取目录失败")
        .map(|e| e.expect("目录项读取失败").file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// 核心断言：同一组输入分别喂给各个 Adapter，要求**报告相同、文件系统结果相同**。
///
/// 返回清理报告（各者相等，取其一）。
fn assert_identical_cleanup(
    root: &Path,
    tag: &str,
    entries: &[&str],
    active: &[&str],
) -> CleanupReport {
    // 各 Adapter 的 base_dir 语义对齐：都是「配置根目录」，skills 在其下。
    //   Antigravity: <base>/skills  ← 生产上是 ~/.gemini/config/skills
    //   Claude Code: <base>/skills  ← 生产上是 ~/.claude/skills
    //   Codex:   <base>/skills  ← 生产上是 ~/.codex/skills
    let gemini_base = root.join(format!("{tag}-gemini"));
    let claude_base = root.join(format!("{tag}-claude"));
    let codex_base = root.join(format!("{tag}-codex"));
    let gemini_skills = gemini_base.join("skills");
    let claude_skills = claude_base.join("skills");
    let codex_skills = codex_base.join("skills");

    make_tree(&gemini_skills, entries);
    make_tree(&claude_skills, entries);
    make_tree(&codex_skills, entries);

    let active_owned: Vec<String> = active.iter().map(|s| (*s).to_string()).collect();

    let from_antigravity = AntigravityAdapter::with_base_dir(&gemini_base)
        .cleanup_orphans(&active_owned)
        .expect("Antigravity 清理应成功");
    let from_claude = ClaudeCodeAdapter::with_base_dir(&claude_base)
        .cleanup_orphans(&active_owned)
        .expect("Claude Code 清理应成功");
    let from_codex = CodexAdapter::with_base_dir(&codex_base)
        .cleanup_orphans(&active_owned)
        .expect("Codex 清理应成功");

    assert_eq!(
        from_antigravity, from_claude,
        "[{tag}] Antigravity 与 Claude Code 的清理报告不一致"
    );
    assert_eq!(
        from_antigravity, from_codex,
        "[{tag}] Codex 与 Antigravity 的清理报告不一致"
    );
    assert_eq!(
        list_sorted(&gemini_skills),
        list_sorted(&claude_skills),
        "[{tag}] Antigravity 与 Claude Code 清理后的目录树不一致"
    );
    assert_eq!(
        list_sorted(&gemini_skills),
        list_sorted(&codex_skills),
        "[{tag}] Codex 与 Antigravity 清理后的目录树不一致"
    );

    from_antigravity
}

// ============================================================================
// TASK-08 验收用例
// ============================================================================

/// 验收第 1 条 + 第 3 条：两个 Adapter 各自都做了孤儿清理，且行为一致。
#[test]
fn t8_both_delete_orphans_and_keep_active() {
    let root = TempRoot::new("orphans");
    let report = assert_identical_cleanup(
        root.path(),
        "orphans",
        &[
            "d:crossbrain-vue3-7c8e2a",
            "d:crossbrain-rust-11aa22",
            "d:crossbrain-gone-99ff88",
        ],
        &["crossbrain-vue3-7c8e2a"],
    );

    assert_eq!(
        report.deleted_dirs,
        vec!["crossbrain-gone-99ff88", "crossbrain-rust-11aa22"],
        "不在 active 列表中的两个目录都应被删除"
    );
    assert_eq!(report.kept_dirs, vec!["crossbrain-vue3-7c8e2a"]);

    // 文件系统层面复核：只剩下 active 的那一个
    let skills = root.path().join("orphans-gemini").join("skills");
    assert_eq!(list_sorted(&skills), vec!["crossbrain-vue3-7c8e2a"]);
}

/// 验收第 3 条：**两者**都不得误删非 `crossbrain-` 前缀的目录。
/// 用本机真实存在的用户自建技能名，避免构造出「实验室里才会有的名字」。
#[test]
fn t8_both_never_touch_non_crossbrain_dirs() {
    let root = TempRoot::new("userdirs");
    let report = assert_identical_cleanup(
        root.path(),
        "userdirs",
        &[
            "d:grill-me",
            "d:design-taste-frontend",
            "d:crossbrain-old-000001",
        ],
        &[], // active 为空 = 最激进场景：CrossBrain 的东西全清
    );

    assert_eq!(report.deleted_dirs, vec!["crossbrain-old-000001"]);
    assert!(
        report.kept_dirs.is_empty(),
        "非 crossbrain- 前缀的目录不应出现在报告里：{:?}",
        report.kept_dirs
    );

    // 用户技能必须原封不动地留在盘上（含其内部文件）
    let skills = root.path().join("userdirs-gemini").join("skills");
    assert_eq!(list_sorted(&skills), vec!["design-taste-frontend", "grill-me"]);
    assert!(
        skills.join("grill-me").join("SKILL.md").is_file(),
        "用户技能的内部文件被破坏"
    );
}

/// 验收第 4 条：目标 skills 目录不存在时**两者**都返回 `Ok`（不报错、不 panic）。
///
/// 这是首次运行的常见状态：用户装了工具但从没同步过，`.gemini/config/skills/`
/// 根本不存在。「目录不存在」是**正常情况**，不是错误——报错会让首次向导中断。
#[test]
fn t8_both_return_ok_when_skills_dir_missing() {
    let root = TempRoot::new("nodir");
    let gemini_base = root.path().join("missing-gemini");
    let claude_base = root.path().join("missing-claude");
    // 刻意只创建 base_dir，不创建其下的 skills/
    fs::create_dir_all(&gemini_base).unwrap();
    fs::create_dir_all(&claude_base).unwrap();

    let active = vec!["crossbrain-anything-123456".to_string()];
    let a = AntigravityAdapter::with_base_dir(&gemini_base).cleanup_orphans(&active);
    let c = ClaudeCodeAdapter::with_base_dir(&claude_base).cleanup_orphans(&active);

    assert!(a.is_ok(), "Antigravity 在 skills 目录缺失时应返回 Ok：{a:?}");
    assert!(c.is_ok(), "Claude Code 在 skills 目录缺失时应返回 Ok：{c:?}");

    let empty = CleanupReport {
        deleted_dirs: Vec::new(),
        kept_dirs: Vec::new(),
    };
    assert_eq!(a.unwrap(), empty, "目录缺失时应返回空报告");
    assert_eq!(c.unwrap(), empty, "目录缺失时应返回空报告");

    // 连带确认：清理动作不能把 skills 目录「顺手创建」出来
    assert!(!gemini_base.join("skills").exists());
    assert!(!claude_base.join("skills").exists());
}

/// 验收第 2 条的补强：`crossbrain-` 前缀的**普通文件**不能被当成目录删除。
///
/// 对普通文件调 `remove_dir_all` 必然报错；若实现不区分类型，
/// 用户恰好手工建了个同名文件就会被绊住，整轮清理以失败告终。
#[test]
fn t8_both_skip_plain_files_with_matching_prefix() {
    let root = TempRoot::new("plainfile");
    let report = assert_identical_cleanup(
        root.path(),
        "plainfile",
        &[
            "f:crossbrain-not-a-dir-aaaaaa",
            "d:crossbrain-real-dir-bbbbbb",
        ],
        &[],
    );

    assert_eq!(
        report.deleted_dirs,
        vec!["crossbrain-real-dir-bbbbbb"],
        "只有目录才该被删除"
    );

    // 那个同名前缀的文件必须还在，且仍然是文件
    let leftover = root
        .path()
        .join("plainfile-gemini")
        .join("skills")
        .join("crossbrain-not-a-dir-aaaaaa");
    assert!(leftover.is_file(), "同名前缀的普通文件被误删了");
}

/// 全部 active → 无孤儿。用于验证「不该删的一个都不能删」。
#[test]
fn t8_both_keep_everything_when_all_active() {
    let root = TempRoot::new("allactive");
    let report = assert_identical_cleanup(
        root.path(),
        "allactive",
        &["d:crossbrain-a-111111", "d:crossbrain-b-222222"],
        &["crossbrain-a-111111", "crossbrain-b-222222"],
    );

    assert!(report.deleted_dirs.is_empty(), "不应删除任何目录");
    assert_eq!(
        report.kept_dirs,
        vec!["crossbrain-a-111111", "crossbrain-b-222222"]
    );
}

/// 报告必须**确定性有序**：`read_dir` 的顺序由文件系统决定，
/// 若直接透传，UI 每次展示的次序都不同，测试也无法稳定断言。
#[test]
fn t8_both_report_is_deterministically_sorted() {
    let root = TempRoot::new("sorted");
    // 刻意按非字典序创建，排除「碰巧因为创建顺序而对」
    let report = assert_identical_cleanup(
        root.path(),
        "sorted",
        &[
            "d:crossbrain-zebra-000000",
            "d:crossbrain-alpha-000000",
            "d:crossbrain-mike-000000",
        ],
        &[],
    );

    assert_eq!(
        report.deleted_dirs,
        vec![
            "crossbrain-alpha-000000",
            "crossbrain-mike-000000",
            "crossbrain-zebra-000000",
        ],
        "报告必须按名字排序"
    );
}

/// 通过 `Box<dyn Adapter>` 调用时同样一致——这是 TASK-13 同步流程的真实调用形态。
///
/// 前面的用例都是「具名类型分别调用」，本条补上 trait 抽象这一层：
/// 若某个 Adapter 覆写了 `cleanup_orphans` 并偏离共享实现，
/// 只有这条测试会以「上层统一调度」的视角暴露它。
#[test]
fn t8_trait_objects_share_identical_behaviour() {
    let root = TempRoot::new("traitobj");
    let gemini_base = root.path().join("traitobj-gemini");
    let claude_base = root.path().join("traitobj-claude");
    let codex_base = root.path().join("traitobj-codex");
    let entries = [
        "d:crossbrain-keep-111111",
        "d:crossbrain-gone-222222",
        "d:user-skill",
    ];
    make_tree(&gemini_base.join("skills"), &entries);
    make_tree(&claude_base.join("skills"), &entries);
    make_tree(&codex_base.join("skills"), &entries);

    // 上层同步流程持有的就是这个形态
    let adapters: Vec<Box<dyn Adapter>> = vec![
        Box::new(AntigravityAdapter::with_base_dir(&gemini_base)),
        Box::new(ClaudeCodeAdapter::with_base_dir(&claude_base)),
        Box::new(CodexAdapter::with_base_dir(&codex_base)),
    ];

    let active = vec!["crossbrain-keep-111111".to_string()];
    let reports: Vec<CleanupReport> = adapters
        .iter()
        .map(|a| a.cleanup_orphans(&active).expect("清理应成功"))
        .collect();

    // 遍历而非硬编码索引：日后新增 Adapter 时这条断言会提醒把它加进来
    assert_eq!(reports.len(), 3, "对比列表应覆盖全部 Adapter");
    for (i, report) in reports.iter().enumerate().skip(1) {
        assert_eq!(
            &reports[0], report,
            "第 {i} 个 Adapter 经 trait object 调度时行为偏离了共享实现"
        );
    }
    assert_eq!(reports[0].deleted_dirs, vec!["crossbrain-gone-222222"]);
    assert_eq!(reports[0].kept_dirs, vec!["crossbrain-keep-111111"]);
}
