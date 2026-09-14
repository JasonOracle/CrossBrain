//! commands.rs — 前端与 Rust 之间**唯一**的 IPC 入口。
//!
//! # 为什么需要这一层（文档没写，但绕不过去）
//!
//! `TASK_BREAKDOWN.md` 的 TASK-10~14 直接描述 UI 交互，却没有任何任务交代
//! 「前端怎么调用 Rust 能力」。而 `adapters/` 与 `sync.rs` 都是普通 Rust 模块，
//! 前端根本看不见它们——必须由本模块用 `#[tauri::command]` 逐个暴露。
//!
//! 这也是 UI 任务真正的前置：没有它，`invoke` 无一可用。
//!
//! # 设计约定
//!
//! - **只做转发，不做业务**：参数校验、流程编排都在下层模块，
//!   本层不得出现「顺手在这里也实现一遍」的逻辑——那必然与 `sync.rs` 分叉。
//! - **返回给前端的错误必须是用户可读文案**（`PRD.md` 第 8 节铁律）：
//!   绝不把 `os error 5` 这类系统错误抛到界面上。
//! - **命名**：Rust 侧 `snake_case`，经 serde 转成前端习惯的 `camelCase`。

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::state;
use crate::sync::{self, BackupInfo, SyncReport, ToolInfo};

/// 应用启动时需要知道的全部状态。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupState {
    /// 是否应展示首次运行向导。
    ///
    /// 判定为「否」的两种情况（满足其一即可）：
    /// 1. 用户走完过向导（含在步骤 5 点「跳过，直接进入」）——
    ///    此情形必须单独记录，因为用户可能一条规则都没写，
    ///    只看 `rules.md` 是判断不出来的（见 `state.rs` 模块文档）。
    /// 2. `global/rules.md` 已有非空白内容——说明用户已经在用它了。
    pub first_run: bool,

    /// 上次同步时间（`YYYY-MM-DD HH:MM`），从未同步过则为 `None`
    pub last_sync_at: Option<String>,
    /// 上次同步是否全部成功
    pub last_sync_ok: bool,

    /// 「同步会断开文件共享关系」的说明是否已展示过（TASK-19 / ADR-15）。
    ///
    /// 前端据此决定要不要在首次同步前弹一次说明。放在启动状态里而不是单开一条命令，
    /// 是因为它随应用启动就被需要（两个视图都可能触发首次同步），
    /// 多一次 IPC 往返只会让「点同步 → 先弹说明」多一个可能出现空档的时机。
    pub link_notice_shown: bool,
}

/// 读取启动状态（向导是否显示、上次同步信息）。
#[tauri::command]
pub fn get_startup_state() -> StartupState {
    let app_state = state::load();

    // 读不出规则文件时按「没有规则」处理：那只意味着多显示一次向导，
    // 比因为一个读取错误把用户挡在向导里要好。
    let has_rules = sync::read_global_rules()
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);

    StartupState {
        first_run: !app_state.wizard_completed && !has_rules,
        last_sync_at: app_state.last_sync_at,
        last_sync_ok: app_state.last_sync_ok,
        link_notice_shown: app_state.link_notice_shown,
    }
}

/// 检测各 AI 工具是否已安装（向导步骤 2）。
#[tauri::command]
pub fn detect_tools() -> Vec<ToolInfo> {
    sync::detect_tools()
}

/// 读取全局规则正文（规则编辑器 / 向导步骤 3 回填）。
#[tauri::command]
pub fn read_global_rules() -> Result<String, String> {
    sync::read_global_rules()
}

/// 保存全局规则正文。
///
/// ⚠️ **保存不触发同步**——「编辑」与「同步」是两个独立动作（`PRD.md` 第 6.1 节）。
/// 想同步请另外调用 [`run_sync`]。
#[tauri::command]
pub fn save_global_rules(content: String) -> Result<(), String> {
    let path = crate::paths::global_rules_file();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| "无法创建配置目录，请手动创建后重试".to_string())?;
    }

    std::fs::write(&path, content)
        .map_err(|_| "没有写入权限，请检查目录访问权限".to_string())
}

/// 标记首次运行向导已完成（含用户主动跳过）。
#[tauri::command]
pub fn complete_wizard() -> Result<(), String> {
    state::mark_wizard_completed().map_err(|_| "无法保存设置，请检查磁盘权限".to_string())
}

/// 执行完整同步（向导步骤 4 / 主界面「立即同步」）。
///
/// # 为什么是 `async` + `spawn_blocking`
///
/// 同步是**阻塞式文件 IO**（可能持续数百毫秒）。若让它在主线程上跑，
/// 主线程被占住 → Vue 收得到事件也**渲染不出来** →
/// 「同步进度逐行实时显示」就退化成了假实时（TASK-10 的硬性验收项）。
/// 因此必须挪到阻塞线程池，把主线程让给 UI。
///
/// 进度经 [`sync::SYNC_PROGRESS_EVENT`] 事件逐个工具推送，
/// 而不是等全部完成再一次性返回。
#[tauri::command]
pub async fn run_sync(app: AppHandle) -> SyncReport {
    // 事件发送需要 AppHandle 在闭包内可用；app 本身还要留一份写状态，故先 clone
    let emitter = app.clone();

    let joined = tauri::async_runtime::spawn_blocking(move || {
        sync::run_full_sync_with_progress(&mut |progress| {
            // 窗口可能已被关闭——发不出去就算了，不该让同步失败
            let _ = emitter.emit(sync::SYNC_PROGRESS_EVENT, &progress);
        })
    })
    .await;

    let report = match joined {
        Ok(report) => report,
        // 只在同步线程 panic 时走到这里。同步内容是「尽力而为」，
        // 但必须让用户看到明确的失败提示，而不是一个转不完的加载圈。
        Err(_) => SyncReport {
            tools: Vec::new(),
            rules_synced: false,
            skill_count: 0,
            ok: false,
            error: Some("同步过程意外中断，请重试。".to_string()),
        },
    };

    // 记录同步时间供状态栏展示。状态写不进去不影响同步结果本身。
    let _ = state::mark_synced(sync::now_display(), report.ok);

    report
}

/// 查询各工具的备份现状（「设置 → 备份与还原」区展示）。
///
/// 只返回**有备份概念**的工具（Claude Code / Codex）——它们才会改动用户的既有文件。
#[tauri::command]
pub fn list_backups() -> Vec<BackupInfo> {
    sync::list_backups()
}

/// 把一个工具的文件还原到首次同步前的原始内容。
///
/// # 这是**用户显式发起**的动作，绝不自动触发
///
/// `ADR-15`：还原必须由用户按下去，程序不替他推断「他大概想还原了」。
/// 成功时返回面向用户的说明文案（含「还原前的内容存到哪了」），
/// 失败时返回面向用户的原因——两者都经 `sync.rs` 转成可读中文。
#[tauri::command]
pub fn restore_backup(tool_id: String) -> Result<String, String> {
    sync::restore_backup(&tool_id)
}

/// 标记「同步会断开文件共享关系」的说明已展示过（只提示一次）。
#[tauri::command]
pub fn mark_link_notice_shown() -> Result<(), String> {
    state::mark_link_notice_shown().map_err(|_| "无法保存设置，请检查磁盘权限".to_string())
}
