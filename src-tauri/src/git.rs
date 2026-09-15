//! git.rs — SSOT 本地版本历史（TASK-09）。
//!
//! 对 `~/.ai-profile/` 做 `git init`（幂等）+ 同步末尾自动提交，
//! 让用户的规则与知识库拥有本地版本历史（`PRD.md` 第 4 节：零网络依赖功能）。
//!
//! # 降级红线（不可违反）
//!
//! 1. **git 未安装 → 静默降级**：返回 `Ok(())`，绝不阻断同步（T8-1）。
//! 2. **提交失败不得影响同步结果**：调用方（`sync.rs`）把 `Err` 转成
//!    报告里的非阻塞提示，同步状态本身不变——内容已经落盘，
//!    版本历史只是附带的便利功能。
//! 3. **断网不影响提交**：全程只调本地 git（T7-3），没有任何网络操作。
//!
//! # 测试注入
//!
//! 生产入口 [`commit_sync`] 固定用 `paths::profile_root()` 与 `"git"`；
//! 测试走 [`commit_with`] 注入临时目录与假程序名（模拟 git 缺失），
//! 保证**真实用户数据零接触**。
//!
//! V1 用 `std::process::Command` 调系统 git；V2 计划迁移 `git2` crate。

use std::path::Path;
use std::process::Command;

use chrono::Local;

use crate::paths;

/// 内置的提交身份兜底：用户没配 git 身份时用 `-c` 临时注入，
/// **不写**进任何 git 配置文件（全局或仓库），不留痕。
const FALLBACK_IDENTITY: [(&str, &str); 2] = [
    ("user.name", "CrossBrain"),
    ("user.email", "crossbrain@local"),
];

/// 同步末尾调用：保证仓库存在并提交全部变更。
///
/// - git 不可用 → `Ok(())`（降级，不报错）
/// - 没有变更 → `Ok(())`（nothing to commit 视为成功）
/// - 真失败（add/init 出错等）→ `Err`，由调用方降级为非阻塞提示
pub fn commit_sync() -> Result<(), String> {
    if !is_git_available() {
        return Ok(());
    }
    commit_with("git", &paths::profile_root())
}

/// 可注入核心。`git_program` 传一个不存在的程序名即可模拟「git 未安装」。
fn commit_with(git_program: &str, profile: &Path) -> Result<(), String> {
    // ── 1. 可用性探测：失败即降级成功（T8-1）──
    if !git_works(git_program) {
        return Ok(());
    }

    // ── 2. 幂等建仓：已有仓库（含 worktree 场景的 .git 文件）就跳过 ──
    if !profile.join(".git").exists() {
        run(git_program, profile, &["init"], &[])?;
    }

    // ── 3. 全量暂存 ──
    run(git_program, profile, &["add", "-A"], &[])?;

    // ── 4. 提交：时间戳走 chrono（铁律，绝不调 shell date）──
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();
    let message = format!("sync: {timestamp}");

    // 身份兜底：仅当用户没配过 user.email 时注入 `-c`，不碰任何配置文件
    let has_identity = Command::new(git_program)
        .args(["config", "--get", "user.email"])
        .current_dir(profile)
        .output()
        .map(|o| o.status.success() && !o.stdout.iter().all(|b| b.is_ascii_whitespace()))
        .unwrap_or(false);
    let identity_args: Vec<&str> = if has_identity {
        Vec::new()
    } else {
        FALLBACK_IDENTITY
            .iter()
            .flat_map(|(k, v)| [*k, *v])
            .collect()
    };

    run(
        git_program,
        profile,
        &["commit", "-m", &message],
        &identity_args,
    )
    // "nothing to commit" 是正常情况：没有变更也视为成功（验收项）
    .or_else(|e| {
        if e.contains("nothing to commit") {
            Ok(())
        } else {
            Err(e)
        }
    })
}

/// git 是否可用（能否执行 `--version`）。
pub fn is_git_available() -> bool {
    git_works("git")
}

fn git_works(git_program: &str) -> bool {
    Command::new(git_program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 在 `dir` 里跑一条 git 命令；`-c` 附加参数用于临时注入配置。
fn run(git_program: &str, dir: &Path, args: &[&str], extra: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new(git_program);
    cmd.args(extra).args(args).current_dir(dir);
    let output = cmd
        .output()
        .map_err(|e| format!("无法执行 git：{e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Windows 上部分 git 输出走 stdout（如 nothing to commit 的状态提示）
    Err(if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 进程号 + 原子计数器唯一命名（与 `state.rs` 同款基座，零外部依赖）。
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "crossbrain-git-{tag}-{}-{n}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("创建临时目录失败");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // Windows 句柄延迟释放可能导致删除失败，忽略即可
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// 不存在的程序名 → 静默降级 Ok（T8-1 的自动化形态）。
    #[test]
    fn missing_git_degrades_to_ok() {
        let tmp = TempDir::new("nogit");
        let result = commit_with("definitely-not-a-real-git-crossbrain", &tmp.path);
        assert_eq!(result, Ok(()));
        assert!(!tmp.path.join(".git").exists(), "降级时不得建仓");
    }

    /// 全流程：建仓 → 提交 → log 里有 `sync: <ISO-8601>`（T8-3）→
    /// 二次提交静默成功。
    #[test]
    fn commit_flow_creates_history_then_stays_quiet() {
        let tmp = TempDir::new("flow");
        let profile = &tmp.path;
        fs::create_dir_all(profile.join("global")).unwrap();
        fs::write(profile.join("global").join("rules.md"), "# 我的规则").unwrap();

        // 首次：建仓 + 提交
        commit_with("git", profile).unwrap();
        assert!(profile.join(".git").exists(), "仓库未创建");

        let log = git_output("git", profile, &["log", "-1", "--pretty=%s"]);
        assert!(
            log.starts_with("sync: "),
            "commit message 应以 'sync: ' 开头，实际：{log}"
        );
        // ISO-8601：`sync: 2026-09-15T14:20:00+0800`（19 字符日期时间 + 5 字符时区）
        let stamp = log.trim_start_matches("sync: ").trim();
        assert_eq!(stamp.len(), 24, "时间戳长度应为 24（ISO-8601 带时区）：{stamp}");
        assert!(
            stamp.as_bytes()[4] == b'-' && stamp.as_bytes()[10] == b'T',
            "时间戳格式应为 YYYY-MM-DDTHH:MM:SS+ZZZZ：{stamp}"
        );

        // 二次：无变更 → nothing to commit → 仍 Ok
        let head_before = git_output("git", profile, &["rev-parse", "HEAD"]);
        commit_with("git", profile).unwrap();
        let head_after = git_output("git", profile, &["rev-parse", "HEAD"]);
        assert_eq!(head_before, head_after, "无变更时不得产生新提交");

        // 三次：有新变更 → 新提交
        fs::write(profile.join("knowledge-test.md"), "新知识条目").unwrap();
        commit_with("git", profile).unwrap();
        let count = git_output("git", profile, &["rev-list", "--count", "HEAD"]);
        assert_eq!(count.trim(), "2", "新增文件后应有第 2 个提交");
    }

    /// 幂等：仓库已存在时不得报错、不得重建。
    #[test]
    fn commit_is_idempotent_when_repo_exists() {
        let tmp = TempDir::new("idem");
        let profile = &tmp.path;
        commit_with("git", profile).unwrap();
        let git_dir = profile.join(".git");
        assert!(git_dir.exists());
        // 再来两次也不出错（目录已存在、无变更）
        commit_with("git", profile).unwrap();
        commit_with("git", profile).unwrap();
        assert!(git_dir.exists());
    }

    /// 提交后用户数据（rules.md 内容）必须原样在历史与工作区中。
    #[test]
    fn commit_preserves_content() {
        let tmp = TempDir::new("content");
        let profile = &tmp.path;
        fs::write(profile.join("rules.md"), "内容 A").unwrap();
        commit_with("git", profile).unwrap();
        fs::write(profile.join("rules.md"), "内容 B").unwrap();
        commit_with("git", profile).unwrap();
        assert_eq!(
            fs::read_to_string(profile.join("rules.md")).unwrap(),
            "内容 B",
            "工作区内容不得被 git 操作改动"
        );
        let show = git_output("git", profile, &["show", "HEAD~1:rules.md"]);
        assert_eq!(show.trim(), "内容 A", "历史版本应可回溯");
    }

    /// .gitignore 里登记的条目（AGENTS.md、状态文件）不得进版本历史。
    #[test]
    fn commit_respects_ssot_gitignore() {
        let tmp = TempDir::new("ignore");
        let profile = &tmp.path;
        // 生产环境里 .gitignore 由 init.rs 的 ensure_profile_dirs 保证存在，
        // 这里等价模拟（条目与 init.rs 的 SSOT_GITIGNORE_ENTRIES 一致）
        fs::write(
            profile.join(".gitignore"),
            format!("AGENTS.md\n{}\n", paths::STATE_FILE_NAME),
        )
        .unwrap();
        fs::write(profile.join("AGENTS.md"), "动态中间产物").unwrap();
        fs::write(profile.join(paths::STATE_FILE_NAME), "{}").unwrap();
        fs::write(profile.join("rules.md"), "真正的内容").unwrap();
        commit_with("git", profile).unwrap();
        let tracked = git_output("git", profile, &["ls-files"]);
        assert!(tracked.contains("rules.md"), "rules.md 应被跟踪");
        assert!(!tracked.contains("AGENTS.md"), "AGENTS.md 不得被跟踪：{tracked}");
        assert!(
            !tracked.contains(".crossbrain-state"),
            "状态文件不得被跟踪：{tracked}"
        );
    }

    fn git_output(program: &str, dir: &Path, args: &[&str]) -> String {
        let out = Command::new(program)
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?} 执行失败：{e}"));
        assert!(
            out.status.success(),
            "git {args:?} 失败：{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }
}
