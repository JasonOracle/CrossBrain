pub mod adapters;
pub mod commands;
mod init;
// `pub` 供 examples/ 下的干跑工具定位真实目录（它需要算出 home 来构造副本）
pub mod paths;
pub mod slug;
mod state;
pub mod sync;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 应用启动时立即初始化 SSOT 目录底座（~/.ai-profile/）。
    // 失败不 panic：目录不可用时应用仍应能打开并给出提示，而不是闪退。
    if let Err(e) = init::ensure_profile_dirs() {
        eprintln!("CrossBrain 初始化失败：{}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_startup_state,
            commands::detect_tools,
            commands::read_global_rules,
            commands::save_global_rules,
            commands::complete_wizard,
            commands::run_sync,
            commands::list_backups,
            commands::restore_backup,
            commands::mark_link_notice_shown,
            commands::list_knowledge,
            commands::read_knowledge,
            commands::create_knowledge,
            commands::save_knowledge,
            commands::delete_knowledge,
            commands::uninstall_crossbrain,
            commands::run_tool_probe,
            commands::remove_tool_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
