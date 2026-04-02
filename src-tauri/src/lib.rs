mod analyzer;
mod brief;
mod commands;
mod db;
mod parser;
mod tracker;

use db::Db;
use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn show_capture_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("capture") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(app, "capture", WebviewUrl::App("/capture".into()))
            .title("")
            .inner_size(600.0, 64.0)
            .decorations(false)
            .always_on_top(true)
            .center()
            .resizable(false)
            .skip_taskbar(true)
            .visible(true)
            .focused(true)
            .build();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let conn = db::init(&app_data_dir).expect("failed to initialize database");
            app.manage(Db(Mutex::new(conn)));

            // Start background focus tracker
            tracker::start(app.handle().clone());

            // Register global shortcut: CmdOrCtrl+Shift+L
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyL);
            let handle = app.handle().clone();
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    show_capture_window(&handle);
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_log,
            commands::get_logs,
            commands::close_capture,
            commands::get_settings,
            commands::set_tracking_paused,
            commands::set_capture_titles,
            commands::complete_onboarding,
            commands::get_excluded_apps,
            commands::exclude_app,
            commands::remove_app_exclusion,
            commands::get_focus_events,
            commands::get_focus_event_count,
            commands::delete_all_data,
            commands::end_waiting,
            commands::get_waiting_periods,
            commands::get_active_waiting_periods,
            commands::recompute_clusters,
            commands::get_clusters,
            commands::get_cluster_detail,
            commands::update_cluster_status,
            commands::update_cluster_title,
            commands::update_cluster_category,
            commands::update_log_category,
            commands::get_review_stats,
            commands::get_clusters_for_review,
            commands::generate_brief,
            commands::get_brief,
            commands::get_brief_by_cluster,
            commands::update_brief,
            commands::mark_brief_exported,
            commands::update_brief_outcome,
            commands::get_briefs_needing_followup,
            commands::get_wins,
            commands::get_total_savings_mins,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
