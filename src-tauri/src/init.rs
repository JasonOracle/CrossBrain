//! init.rs — 应用启动时初始化 `~/.ai-profile/` 目录底座（SSOT）。
//!
//! 本模块只负责「保证底座存在」，**全部为幂等操作**：
//! 已存在的目录和文件一律原样保留，不覆盖、不报错。
//! 因此反复启动应用是安全的，且绝不会动到用户已写好的内容。

use std::fs;
use std::path::Path;

use crate::paths;

/// SSOT 的 `.gitignore` 必须包含的条目（缺哪条补哪条，已有内容一律不动）。
///
/// - `AGENTS.md`：每次同步前重新生成的动态中间产物，不应纳入本地版本历史
///   （`docs/tech/ARCHITECTURE.md` 第 2 节「SSOT 本地数据仓库结构」）。
/// - 状态文件：本机运行状态（向导进度、上次同步时间）。
///   它描述的是**这台机器**的状态，纳入版本历史会在多机同步时互相冲突。
///   名字取自 [`paths::STATE_FILE_NAME`]，**不在此处另写一份字符串**——
///   改名时两处不同步的后果是状态文件悄悄被纳入版本历史。
const SSOT_GITIGNORE_ENTRIES: [&str; 2] = ["AGENTS.md", paths::STATE_FILE_NAME];

/// 初始化 SSOT 目录结构。幂等操作，已存在不报错。
///
/// 在 Tauri `Builder` 启动前调用（见 `lib.rs`）。
///
/// # 失败处理
///
/// 返回 `Err` 时调用方只记录日志、**不 panic**：
/// 目录建不出来（如权限受限）时应用仍应能打开并给出提示，
/// 而不是直接崩溃让用户看不到任何信息。
pub fn ensure_profile_dirs() -> Result<(), String> {
    // 1. 目录底座 —— create_dir_all 自带「创建父级 + 已存在则成功」语义
    let dirs_to_create = [
        paths::profile_root(),
        paths::profile_root().join("global"),
        paths::knowledge_dir(),
    ];

    for dir in dirs_to_create {
        fs::create_dir_all(&dir).map_err(|e| format!("无法创建目录 {}：{}", dir.display(), e))?;
    }

    // 2. 全局规则文件 —— 仅在缺失时创建空文件（首次启动场景）
    let rules_file = paths::global_rules_file();
    if !rules_file.exists() {
        fs::write(&rules_file, "").map_err(|e| format!("无法创建 rules.md：{}", e))?;
    }

    // 3. SSOT 的 .gitignore —— 只补齐缺失条目，绝不重写已有内容
    ensure_gitignore(&paths::profile_root().join(".gitignore"))?;

    Ok(())
}

/// 幂等地保证 `.gitignore` 含全部必需条目。
///
/// # 为什么不直接「文件不存在就写死一份内容」
///
/// TASK-03 阶段只写了 `AGENTS.md` 一条；后来状态文件也需要被忽略。
/// 若直接覆盖，用户自己加的忽略规则会被删掉——那是用户的文件，不是我们的产物。
///
/// 因此采用**逐行补齐**：已含的行不动，缺的行追加；一条都不缺时**完全跳过写盘**
/// （连 mtime 都不变——这是 TASK-03「二次启动零变化」实测的断言之一）。
fn ensure_gitignore(path: &Path) -> Result<(), String> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();

    let missing: Vec<&str> = SSOT_GITIGNORE_ENTRIES
        .iter()
        .filter(|entry| !lines.iter().any(|line| line.trim() == **entry))
        .copied()
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    lines.extend(missing.into_iter().map(str::to_string));
    let content = format!("{}\n", lines.join("\n"));
    fs::write(path, content).map_err(|e| format!("无法更新 .gitignore：{}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// SSOT 目录底座必须真实落盘，且重复调用不报错（幂等）。
    #[test]
    fn ensure_profile_dirs_is_idempotent() {
        // 先记录：若测试前已存在，不因为测试而破坏用户数据
        let root = paths::profile_root();
        let rules = paths::global_rules_file();
        let pre_existing_rules = fs::read_to_string(&rules).ok();

        ensure_profile_dirs().expect("首次初始化应成功");
        ensure_profile_dirs().expect("重复初始化必须幂等成功");

        assert!(root.is_dir(), "profile 根目录未创建");
        assert!(root.join("global").is_dir(), "global/ 未创建");
        assert!(paths::knowledge_dir().is_dir(), "knowledge/ 未创建");
        assert!(rules.is_file(), "global/rules.md 未创建");
        assert!(root.join(".gitignore").is_file(), ".gitignore 未创建");

        // 幂等性核心断言：已有内容不会被覆盖
        if let Some(before) = pre_existing_rules {
            assert_eq!(
                fs::read_to_string(&rules).unwrap(),
                before,
                "重复初始化覆盖了已有的 rules.md 内容"
            );
        }
    }

    // ---------- .gitignore 补齐（TASK-10 期间补入）----------
    //
    // 这几个用例**必须**注入临时路径：默认路径是真实的 `~/.ai-profile/.gitignore`，
    // 本机有真实用户数据，不能拿它做实验。

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct TempFile {
        path: PathBuf,
        dir: PathBuf,
    }

    impl TempFile {
        fn new(tag: &str) -> Self {
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!(
                "crossbrain-init-{tag}-{}-{n}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("创建临时目录失败");
            Self {
                path: dir.join(".gitignore"),
                dir,
            }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            // Windows 句柄延迟释放可能导致删除失败，忽略即可
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    /// 文件不存在时应写入全部必需条目。
    #[test]
    fn gitignore_created_with_all_entries() {
        let f = TempFile::new("create");
        ensure_gitignore(&f.path).unwrap();

        let content = fs::read_to_string(&f.path).unwrap();
        for entry in SSOT_GITIGNORE_ENTRIES {
            assert!(
                content.lines().any(|l| l.trim() == entry),
                "缺少条目 {entry}：{content:?}"
            );
        }
    }

    /// 只缺部分条目时，必须**追加**而不是覆盖——用户自己写的规则不能丢。
    #[test]
    fn gitignore_preserves_user_entries_and_appends_missing() {
        let f = TempFile::new("append");
        // 模拟 TASK-03 时期的状态：只有 AGENTS.md，且用户自己加了一条
        fs::write(&f.path, "AGENTS.md\n*.bak\n").unwrap();

        ensure_gitignore(&f.path).unwrap();

        let content = fs::read_to_string(&f.path).unwrap();
        assert!(content.contains("*.bak"), "用户的条目被覆盖了：{content:?}");
        assert!(
            content.lines().any(|l| l.trim() == ".crossbrain-state.json"),
            "缺失条目未被追加：{content:?}"
        );
    }

    /// 条目齐全时必须**完全不写盘**：这保证二次启动 mtime 不变（TASK-03 的幂等实测）。
    #[test]
    fn gitignore_untouched_when_complete() {
        let f = TempFile::new("complete");
        fs::write(&f.path, "AGENTS.md\n.crossbrain-state.json\n").unwrap();
        let before = fs::metadata(&f.path).unwrap().modified().unwrap();

        ensure_gitignore(&f.path).unwrap();

        let after = fs::metadata(&f.path).unwrap().modified().unwrap();
        assert_eq!(before, after, "条目齐全时不应重写文件");
    }

    /// 重复调用不得产生重复行。
    #[test]
    fn gitignore_has_no_duplicate_lines() {
        let f = TempFile::new("dupe");
        ensure_gitignore(&f.path).unwrap();
        ensure_gitignore(&f.path).unwrap();

        let content = fs::read_to_string(&f.path).unwrap();
        let count = content
            .lines()
            .filter(|l| l.trim() == ".crossbrain-state.json")
            .count();
        assert_eq!(count, 1, "出现重复条目：{content:?}");
    }
}
