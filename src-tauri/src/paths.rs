//! paths.rs — CrossBrain 所有路径的统一出口。
//!
//! # 规则（铁律 L-02）
//!
//! 任何文件需要路径时，**必须**调用本模块的函数。
//! 禁止在别处自行拼接路径，禁止硬编码盘符（如 `C:\Users\xxx`）或用户名——
//! Windows 用户的 `%USERPROFILE%` 完全可能在 D 盘或自定义路径上。
//!
//! 路径拼接一律使用 [`PathBuf::join`]，不拼接字符串，避免分隔符跨平台不一致。

use std::path::PathBuf;

/// 获取系统 home 目录。
///
/// 本模块唯一一次调用 `dirs::home_dir()` 的地方，集中处理失败分支，
/// 避免每个路径函数里重复 `.expect(...)`。
///
/// 正常情况下此调用不会失败；真失败说明运行环境已异常，
/// 此时任何路径都不可信，属于无法继续的致命错误。
fn home_dir() -> PathBuf {
    dirs::home_dir().expect("无法获取系统 home 目录，这是一个致命错误")
}

/// 获取 `~/.ai-profile/` 根目录（SSOT 单一事实源根目录）。
pub fn profile_root() -> PathBuf {
    home_dir().join(".ai-profile")
}

/// 获取 `~/.ai-profile/global/rules.md`（全局规则，用户编辑的唯一源头）。
pub fn global_rules_file() -> PathBuf {
    profile_root().join("global").join("rules.md")
}

/// 获取 `~/.ai-profile/knowledge/` 目录（技能知识，每篇一个主题）。
pub fn knowledge_dir() -> PathBuf {
    profile_root().join("knowledge")
}

/// 获取 `~/.ai-profile/AGENTS.md`（动态生成的中间产物，每次同步前重建）。
pub fn agents_md_file() -> PathBuf {
    profile_root().join("AGENTS.md")
}

/// 应用运行状态文件名（SSOT 根目录下的隐藏文件）。
///
/// # 为什么定义在这里
///
/// 有三处需要这个名字：读写状态（`state.rs`）、写进 `.gitignore`（`init.rs`）、
/// 以及测试里的临时文件断言。各写一份字符串，改名时必有一处被漏掉——
/// 而漏掉 `init.rs` 那处的后果是状态文件被纳入版本历史，多机同步时互相冲突。
///
/// 前置点号让它在资源管理器里默认隐藏——用户不该手动编辑它。
pub const STATE_FILE_NAME: &str = ".crossbrain-state.json";

/// 获取 `~/.ai-profile/.crossbrain-state.json`（应用运行状态文件）。
///
/// 存的是**本机状态**（向导是否已完成、上次同步时间），不是用户内容：
/// 不参与同步、不进版本历史（已由 `init.rs` 写入 SSOT 的 `.gitignore`）。
pub fn state_file() -> PathBuf {
    profile_root().join(STATE_FILE_NAME)
}

/// 获取 `~/.gemini/config/` 目录（Antigravity IDE 的检测与写入路径）。
pub fn gemini_config_dir() -> PathBuf {
    home_dir().join(".gemini").join("config")
}

/// 获取 `~/.claude/` 目录（Claude Code 的检测与写入路径）。
pub fn claude_dir() -> PathBuf {
    home_dir().join(".claude")
}

/// 获取 `~/.codex/` 目录（Codex 的检测与写入路径）。
///
/// 这是 **Codex 桌面 App 与命令行版（CLI）共用的 Codex home**
/// （App 在 `D:\soft\win-x64`，官方产品名就叫 `Codex`），
/// 环境变量 `CODEX_HOME` 可覆盖，但默认即此目录。
///
/// 内部结构（2026-09-14 本机实测）：
/// - `AGENTS.md` —— L0 落点，CrossBrain 走标记块注入
/// - `AGENTS.override.md` —— 优先级**高于** `AGENTS.md`，它存在会让我们的规则失效
/// - `skills/` —— L2 落点，其中 `skills/.system/` 是 Codex **内置技能**，严禁触碰
/// - `rules/default.rules` —— 命令审批白名单，**不是规则文件**，不要误用
pub fn codex_dir() -> PathBuf {
    home_dir().join(".codex")
}
