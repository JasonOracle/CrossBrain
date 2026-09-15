//! state.rs — 应用运行状态的持久化。
//!
//! # 为什么需要它
//!
//! UI 上有两件事必须**跨启动记住**，而 `global/rules.md` 的内容表达不了它们：
//!
//! 1. **首次运行向导是否已完成**——`PRD.md` 第 5 节把向导的触发条件定义为
//!    「`rules.md` 存在但内容为空」，但用户完全可以在步骤 3 一个字都不写就点
//!    「跳过，直接进入」。此时 `rules.md` 仍为空，下次启动会**再次**弹出向导，
//!    与验收项「向导完成后再次启动，不再显示向导」直接冲突。
//! 2. **上次同步时间**——`PRD.md` 第 7 节的状态栏要显示它，重启后必须还在。
//!
//! 所以需要一个独立的状态文件（`~/.ai-profile/.crossbrain-state.json`）。
//! 它是**本机状态**而非用户内容：不参与同步、不进版本历史。
//!
//! # 容错原则
//!
//! 状态文件**丢失或损坏都不应阻断应用启动**——最坏情况只是重新显示一次向导。
//! 因此 [`load`] 在任何异常下都回退为默认值，**绝不返回 `Err`**；
//! 这是一个刻意的取舍：把「状态文件读不出来」升级为「应用打不开」是错误的优先级。
//! 同理，写入不做原子替换（temp + rename）——写坏的代价是重看一次向导，
//! 不值得为此引入 Windows 上 `rename` 不覆盖目标的额外分支。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths;

/// 需要跨启动记住的应用状态。
///
/// 每个字段都带 `#[serde(default)]`：日后新增字段时，旧版本写下的状态文件
/// 仍能正常解析，缺的字段取默认值，不会让用户因为一次升级就重看向导。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppState {
    /// 首次运行向导是否已完成（含用户主动「跳过」）。
    #[serde(default)]
    pub wizard_completed: bool,

    /// 上次同步的本地时间，展示用字符串（`YYYY-MM-DD HH:MM`）。
    ///
    /// 不做时区解析：它只被直接显示，从未参与计算，
    /// 存成已格式化的字符串反而消除了「读到别的时区」这类问题。
    #[serde(default)]
    pub last_sync_at: Option<String>,

    /// 上次同步是否全部成功（状态栏据此显示 ✅ / ❌）。
    #[serde(default)]
    pub last_sync_ok: bool,

    /// 「同步会断开文件共享关系」的说明是否已展示过（TASK-19 / ADR-15）。
    ///
    /// 首次同步会断开 `~/.claude/CLAUDE.md` 与另外 4 个路径之间的硬链接——
    /// 内容一字不变，但该文件此后不再跟随其余 4 处一起变更。
    /// 用户本来就是靠硬链接实现「一份内容、5 个工具生效」的，这属于改变其既有工作流，
    /// 因此必须**事前**告知（`ADR-15` 的「知情」要求）。
    ///
    /// 用持久化标记而不是每次弹：告知一次即可，反复打断是另一种不尊重。
    #[serde(default)]
    pub link_notice_shown: bool,

    /// 用户在「工具接入」扫描弹窗里勾选保存的工具 id 清单（TASK-21）。
    ///
    /// `None` = 用户**从未用过扫描**（老用户 / 新安装）——此时按默认行为：
    /// 三个已适配工具全部参与同步。这个语义让旧状态文件无需迁移。
    ///
    /// `Some(list)` = 用户明确做过选择。清单里**允许出现尚未适配的工具 id**
    /// （如 `cursor`）：那是用户的接入意愿，先记下来作为后续适配的依据；
    /// 参与同步的只有 `list` 与已适配工具的交集（见 `sync.rs` 的过滤逻辑）。
    #[serde(default)]
    pub selected_tools: Option<Vec<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wizard_completed: false,
            last_sync_at: None,
            last_sync_ok: false,
            link_notice_shown: false,
            selected_tools: None,
        }
    }
}

/// 状态文件的规范路径（测试请改用 [`load_from`] / [`save_to`] 注入临时路径）。
pub fn state_file() -> PathBuf {
    paths::state_file()
}

/// 读取应用状态；文件缺失、损坏、无权限时一律回退为默认值。
pub fn load() -> AppState {
    load_from(&state_file())
}

/// 写入应用状态。
pub fn save(state: &AppState) -> Result<(), String> {
    save_to(&state_file(), state)
}

/// 标记向导已完成（`wizard_completed = true`），保留其余字段。
pub fn mark_wizard_completed() -> Result<(), String> {
    let mut state = load();
    state.wizard_completed = true;
    save(&state)
}

/// 记录一次同步结果（时间 + 成败），保留其余字段。
pub fn mark_synced(at: impl Into<String>, ok: bool) -> Result<(), String> {
    let mut state = load();
    state.last_sync_at = Some(at.into());
    state.last_sync_ok = ok;
    save(&state)
}

/// 标记「文件共享关系会被断开」的说明已展示过（TASK-19 / ADR-15）。
///
/// 用户在对话框上点「继续同步」时才调用——**不是**在对话框弹出的那一刻。
/// 弹出即标记的话，用户若关掉窗口或点了取消，这条说明就永远不会再出现了。
pub fn mark_link_notice_shown() -> Result<(), String> {
    let mut state = load();
    state.link_notice_shown = true;
    save(&state)
}

/// 记录用户在「工具接入」弹窗里勾选保存的工具清单（TASK-21）。
///
/// 允许包含尚未适配的工具 id（那是**意愿记录**，不同步）；
/// 去重与合法性校验由调用方（`sync.rs`）负责，这里只做持久化。
pub fn mark_selected_tools(ids: Vec<String>) -> Result<(), String> {
    let mut state = load();
    state.selected_tools = Some(ids);
    save(&state)
}

/// 删除状态文件（TASK-14「一键卸载 / 去痕」）。
///
/// 卸载成功后由命令层调用：状态是**本机运行痕迹**，去痕要求它一并消失；
/// 删掉后下次启动 `load()` 回退默认值，应用会重新走向导——这正是
/// 「已卸载、重新配置」的正确行为。文件不存在视为已删除。
///
/// ⚠️ 只删状态文件本身。`~/.ai-profile/` 下的 `global/rules.md` 与
/// `knowledge/` 是**用户自己的数据**，卸载不碰它们。
pub fn remove_state_file() -> Result<(), String> {
    remove_state_file_at(&state_file())
}

/// [`remove_state_file`] 的可注入变体（测试用临时路径，与 [`load_from`] / [`save_to`] 同理）。
pub fn remove_state_file_at(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("无法删除运行状态文件，请检查文件夹访问权限".to_string()),
    }
}

// ============================================================================
// 可注入路径的实现（测试专用入口，生产代码走上面的无参封装）
// ============================================================================

/// 读取指定路径的状态文件。任何异常都回退为默认值。
///
/// # 为什么连 `Err` 都不返回
///
/// 调用方（应用启动路径）拿到 `Err` 也无从补救——状态文件读不出来时，
/// 能做的事情与「读到空状态」完全相同：按未完成向导处理。
/// 返回 `Result` 只会让每个调用点都写一遍 `.unwrap_or_default()`。
fn load_from(path: &Path) -> AppState {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// 写入指定路径的状态文件（自动创建父目录）。
fn save_to(path: &Path, state: &AppState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("无法创建状态目录 {}：{}", parent.display(), e))?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("状态序列化失败：{}", e))?;

    fs::write(path, json).map_err(|e| format!("无法写入状态文件 {}：{}", path.display(), e))
}

#[cfg(test)]
mod tests {
    //! 全部测试跑在系统临时目录（`save_to` / `load_from` 注入路径），
    //! **绝不触碰真实的 `~/.ai-profile/`**——本机有真实用户数据。

    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 进程号 + 原子计数器，不依赖时钟（唯一性场景下比时间戳更简单可靠）。
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "crossbrain-state-{tag}-{}-{n}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("创建临时目录失败");
            Self { path }
        }

        fn file(&self) -> PathBuf {
            self.path.join(paths::STATE_FILE_NAME)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // Windows 句柄延迟释放可能导致删除失败，忽略即可
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// 文件不存在时必须回退默认值，而不是报错——首次启动就是这个场景。
    #[test]
    fn missing_file_yields_default() {
        let dir = TempDir::new("missing");
        assert_eq!(load_from(&dir.file()), AppState::default());
    }

    /// 内容损坏时必须回退默认值：手改坏状态文件不应让应用起不来。
    #[test]
    fn corrupted_file_yields_default() {
        let dir = TempDir::new("corrupt");
        fs::write(dir.file(), "{ 这不是合法 JSON").unwrap();
        assert_eq!(load_from(&dir.file()), AppState::default());

        // 合法 JSON 但结构不符（例如被写成了数组）同样回退
        fs::write(dir.file(), "[1,2,3]").unwrap();
        assert_eq!(load_from(&dir.file()), AppState::default());
    }

    /// 写入后读回必须完全一致（往返一致）。
    #[test]
    fn save_then_load_roundtrips() {
        let dir = TempDir::new("roundtrip");
        let state = AppState {
            wizard_completed: true,
            last_sync_at: Some("2026-09-14 22:30".to_string()),
            last_sync_ok: true,
            link_notice_shown: false,
            selected_tools: Some(vec!["claude_code".to_string(), "codex".to_string()]),
        };

        save_to(&dir.file(), &state).unwrap();
        assert_eq!(load_from(&dir.file()), state);
    }

    /// 旧版本状态文件缺少新字段时，必须按字段级默认值补齐而不是整体解析失败。
    #[test]
    fn partial_json_uses_field_defaults() {
        let dir = TempDir::new("partial");
        fs::write(dir.file(), r#"{"wizard_completed": true}"#).unwrap();

        let state = load_from(&dir.file());
        assert!(state.wizard_completed, "已有字段必须被读出");
        assert_eq!(state.last_sync_at, None, "缺失字段应取默认值");
        assert!(!state.last_sync_ok, "缺失字段应取默认值");
        assert!(
            !state.link_notice_shown,
            "缺失的 link_notice_shown 应取默认值 false —— \
             否则老状态文件会让新用户永远看不到断链说明"
        );
        assert!(
            state.selected_tools.is_none(),
            "缺失的 selected_tools 应取默认值 None —— \
             老用户没做过扫描，必须继续按「三个工具全启用」走，不能被当成「一个都没选」"
        );
    }

    /// TASK-21：selected_tools 必须跨读写保持——
    /// 保存后重启应用，用户勾选的工具清单还在，同步范围才稳定。
    #[test]
    fn selected_tools_roundtrips() {
        let dir = TempDir::new("selected-tools");
        let path = dir.file();

        let ids = vec![
            "codex".to_string(),
            "cursor".to_string(), // 未适配工具：意愿记录，允许保存
        ];
        let mut state = load_from(&path);
        state.selected_tools = Some(ids.clone());
        save_to(&path, &state).unwrap();

        let reloaded = load_from(&path);
        assert_eq!(reloaded.selected_tools, Some(ids));
    }

    /// 断链说明的标记必须能跨读写保持：否则每次同步都会再弹一次。
    #[test]
    fn link_notice_flag_roundtrips() {
        let dir = TempDir::new("link-notice");
        let path = dir.file();

        assert!(!load_from(&path).link_notice_shown, "初始应为未展示");

        let mut state = load_from(&path);
        state.link_notice_shown = true;
        save_to(&path, &state).unwrap();

        assert!(
            load_from(&path).link_notice_shown,
            "标记写入后必须读回 true，否则说明会反复弹出"
        );
    }

    /// 父目录不存在时应自动创建（首次启动时 `.ai-profile/` 可能尚未建立）。
    #[test]
    fn save_creates_parent_directory() {
        let dir = TempDir::new("parent");
        let nested = dir.path.join("not").join("yet").join(paths::STATE_FILE_NAME);

        save_to(&nested, &AppState::default()).unwrap();
        assert!(nested.is_file(), "父目录未被自动创建");
    }

    /// 卸载去痕：状态文件删除后读取必须回退默认值（下次启动重新走向导），
    /// 且删除不存在的文件按成功处理（幂等，TASK-14）。
    #[test]
    fn remove_state_file_is_idempotent_and_resets_to_default() {
        let dir = TempDir::new("remove-state");
        let path = dir.path.join(paths::STATE_FILE_NAME);

        save_to(&path, &AppState::default()).unwrap();
        remove_state_file_at(&path).unwrap();
        assert!(!path.exists(), "状态文件应被删除");

        let fresh = load_from(&path);
        assert!(!fresh.wizard_completed, "删除后必须回到「未初始化」默认值");

        // 再删一次：NotFound 按成功处理
        remove_state_file_at(&path).unwrap();
    }
}
