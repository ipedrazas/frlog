use crate::db::{self, Db};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Manager;

pub struct TrackerPaused(pub Arc<AtomicBool>);

pub fn start(app_handle: tauri::AppHandle) {
    let paused = {
        let db = app_handle.state::<Db>();
        let conn = db.0.lock().unwrap();
        let settings = db::get_settings(&conn);
        Arc::new(AtomicBool::new(settings.tracking_paused))
    };
    app_handle.manage(TrackerPaused(paused.clone()));

    thread::spawn(move || {
        let mut last_app: Option<String> = None;
        let mut last_title: Option<String> = None;

        loop {
            let poll_interval = {
                let db = app_handle.state::<Db>();
                let conn = db.0.lock().unwrap();
                db::get_settings(&conn).poll_interval_secs
            };

            thread::sleep(Duration::from_secs(poll_interval));

            if paused.load(Ordering::Relaxed) {
                continue;
            }

            // Get the active window info
            let active = match x_win::get_active_window() {
                Ok(w) => w,
                Err(_) => continue,
            };

            let app_name = if active.info.name.is_empty() {
                active.info.exec_name.clone()
            } else {
                active.info.name.clone()
            };

            if app_name.is_empty() {
                continue;
            }

            let db = app_handle.state::<Db>();
            let conn = db.0.lock().unwrap();

            // Check exclusion
            if db::is_app_excluded(&conn, &app_name).unwrap_or(false) {
                last_app = None;
                last_title = None;
                continue;
            }

            // Check if titles should be captured
            let settings = db::get_settings(&conn);
            let title = if settings.capture_titles {
                let t = active.title.clone();
                if t.is_empty() { None } else { Some(t) }
            } else {
                None
            };

            // Only record on change
            let changed = last_app.as_deref() != Some(&app_name)
                || last_title.as_deref() != title.as_deref();

            if changed {
                let _ = db::insert_focus_event(&conn, &app_name, title.as_deref());
                last_app = Some(app_name);
                last_title = title;
            }
        }
    });
}
