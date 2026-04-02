use crate::analyzer::{self, LogForClustering};
use crate::brief;
use crate::db::{
    self, AutomationBrief, ClusterDetail, Db, FocusEvent, FrictionCluster, LogEntry, ReviewStats,
    Settings, WaitingPeriod, WinEntry,
};
use crate::investigate::{self, ImportResult};
use crate::parser::{self, DotCommand};
use crate::tracker::TrackerPaused;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

// --- Capture ---

#[tauri::command]
pub fn save_log(app: AppHandle, raw_text: String) -> Result<(), String> {
    // Hide the capture window immediately for perceived speed
    if let Some(window) = app.get_webview_window("capture") {
        let _ = window.hide();
    }

    // Parse dot commands
    let parsed = parser::parse(&raw_text);

    // Get current app context and persist
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let app_context = db::get_current_app(&conn).map_err(|e| e.to_string())?;

    let command_label = parsed.command.as_ref().map(|c| c.label());

    // Determine category and confidence
    let (category, confidence) = if let Some(ref cat) = parsed.category {
        // Dot command: high confidence
        (Some(cat.as_str()), Some(1.0))
    } else {
        // Auto-infer from text
        match analyzer::infer_category(&parsed.note_text) {
            Some((cat, conf)) => (Some(cat), Some(conf)),
            None => (None, None),
        }
    };

    let log_id = db::insert_log(
        &conn,
        &raw_text,
        &parsed.note_text,
        command_label,
        category,
        confidence,
        app_context.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    // Handle waiting/blocking state
    if let Some(ref cmd) = parsed.command {
        match cmd {
            DotCommand::Waiting => {
                db::start_waiting_period(&conn, log_id, &parsed.note_text, "waiting")
                    .map_err(|e| e.to_string())?;
            }
            DotCommand::Blocking => {
                db::start_waiting_period(&conn, log_id, &parsed.note_text, "blocking")
                    .map_err(|e| e.to_string())?;
            }
            _ => {}
        }
    }

    // Notify the main window to refresh
    let _ = app.emit("log-saved", ());

    Ok(())
}

#[tauri::command]
pub fn get_logs(app: AppHandle, filter: Option<String>) -> Result<Vec<LogEntry>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    match filter.as_deref() {
        Some("all") | None => db::get_all_logs(&conn).map_err(|e| e.to_string()),
        Some(f) if f.starts_with("cmd:") => {
            db::get_logs_by_command(&conn, &f[4..]).map_err(|e| e.to_string())
        }
        Some(f) if f.starts_with("cat:") => {
            db::get_logs_by_category(&conn, &f[4..]).map_err(|e| e.to_string())
        }
        Some(f) => {
            // Try as category first, then command
            let by_cat = db::get_logs_by_category(&conn, f).map_err(|e| e.to_string())?;
            if !by_cat.is_empty() {
                Ok(by_cat)
            } else {
                db::get_logs_by_command(&conn, f).map_err(|e| e.to_string())
            }
        }
    }
}

#[tauri::command]
pub fn close_capture(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        let _ = window.hide();
    }
    Ok(())
}

// --- Waiting periods ---

#[tauri::command]
pub fn end_waiting(app: AppHandle, direction: String) -> Result<bool, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::end_waiting_period(&conn, &direction).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_waiting_periods(app: AppHandle) -> Result<Vec<WaitingPeriod>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_waiting_periods(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_waiting_periods(app: AppHandle) -> Result<Vec<WaitingPeriod>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_active_waiting_periods(&conn).map_err(|e| e.to_string())
}

// --- Settings ---

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db::get_settings(&conn))
}

#[tauri::command]
pub fn set_tracking_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::set_setting(
        &conn,
        "tracking_paused",
        if paused { "true" } else { "false" },
    )
    .map_err(|e| e.to_string())?;

    let tracker = app.state::<TrackerPaused>();
    tracker.0.store(paused, Ordering::Relaxed);

    Ok(())
}

#[tauri::command]
pub fn set_capture_titles(app: AppHandle, enabled: bool) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::set_setting(
        &conn,
        "capture_titles",
        if enabled { "true" } else { "false" },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::set_setting(&conn, "onboarding_completed", "true").map_err(|e| e.to_string())
}

// --- Privacy / Exclusions ---

#[tauri::command]
pub fn get_excluded_apps(app: AppHandle) -> Result<Vec<String>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_excluded_apps(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exclude_app(app: AppHandle, app_name: String) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::set_app_excluded(&conn, &app_name, true).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_app_exclusion(app: AppHandle, app_name: String) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::remove_exclusion(&conn, &app_name).map_err(|e| e.to_string())
}

// --- Focus events ---

#[tauri::command]
pub fn get_focus_events(app: AppHandle, limit: Option<i64>) -> Result<Vec<FocusEvent>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_focus_events(&conn, limit.unwrap_or(100)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_focus_event_count(app: AppHandle) -> Result<i64, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_focus_event_count(&conn).map_err(|e| e.to_string())
}

// --- Clusters ---

#[tauri::command]
pub fn recompute_clusters(app: AppHandle) -> Result<Vec<FrictionCluster>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Fetch all logs for clustering
    let raw_logs = db::get_logs_for_clustering(&conn).map_err(|e| e.to_string())?;
    let logs: Vec<LogForClustering> = raw_logs
        .into_iter()
        .map(|(id, note_text, category, app_context)| LogForClustering {
            id,
            note_text,
            category,
            app_context,
        })
        .collect();

    // Compute clusters
    let computed = analyzer::compute_clusters(&logs);

    // Clear old clusters and save new ones
    db::clear_clusters(&conn).map_err(|e| e.to_string())?;
    for cluster in &computed {
        db::insert_cluster(
            &conn,
            &cluster.title,
            &cluster.summary,
            &cluster.category,
            cluster.confidence,
            &cluster.log_ids,
        )
        .map_err(|e| e.to_string())?;
    }

    db::get_all_clusters(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_clusters(app: AppHandle) -> Result<Vec<FrictionCluster>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_all_clusters(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_cluster_detail(
    app: AppHandle,
    cluster_id: i64,
) -> Result<Option<ClusterDetail>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_cluster_detail(&conn, cluster_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_cluster_status(
    app: AppHandle,
    cluster_id: i64,
    status: String,
) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_cluster_status(&conn, cluster_id, &status).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_cluster_title(app: AppHandle, cluster_id: i64, title: String) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_cluster_title(&conn, cluster_id, &title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_cluster_category(
    app: AppHandle,
    cluster_id: i64,
    category: String,
) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_cluster_category(&conn, cluster_id, &category).map_err(|e| e.to_string())
}

// --- Review ---

#[tauri::command]
pub fn get_review_stats(
    app: AppHandle,
    since: String,
    until: String,
) -> Result<ReviewStats, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_review_stats(&conn, &since, &until).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_clusters_for_review(
    app: AppHandle,
    show_ignored: bool,
) -> Result<Vec<FrictionCluster>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_clusters_for_review(&conn, !show_ignored).map_err(|e| e.to_string())
}

// --- Log tag editing ---

#[tauri::command]
pub fn update_log_category(app: AppHandle, log_id: i64, category: String) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_log_category(&conn, log_id, &category, 1.0).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_log_text(app: AppHandle, log_id: i64, raw_text: String) -> Result<(), String> {
    let parsed = parser::parse(&raw_text);
    let command_label = parsed.command.as_ref().map(|c| c.label());

    let (category, confidence) = if let Some(ref cat) = parsed.category {
        (Some(cat.as_str()), Some(1.0))
    } else {
        match analyzer::infer_category(&parsed.note_text) {
            Some((cat, conf)) => (Some(cat), Some(conf)),
            None => (None, None),
        }
    };

    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_log_text(
        &conn,
        log_id,
        &raw_text,
        &parsed.note_text,
        command_label,
        category,
        confidence,
    )
    .map_err(|e| e.to_string())
}

// --- Automation briefs ---

#[tauri::command]
pub fn generate_brief(app: AppHandle, cluster_id: i64) -> Result<AutomationBrief, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check if brief already exists for this cluster
    if let Some(existing) =
        db::get_brief_by_cluster(&conn, cluster_id).map_err(|e| e.to_string())?
    {
        return Ok(existing);
    }

    // Get cluster detail
    let detail = db::get_cluster_detail(&conn, cluster_id)
        .map_err(|e| e.to_string())?
        .ok_or("Cluster not found")?;

    // Generate brief
    let generated = brief::generate(&detail);
    let brief_id = db::insert_brief(&conn, &generated).map_err(|e| e.to_string())?;

    db::get_brief(&conn, brief_id)
        .map_err(|e| e.to_string())?
        .ok_or("Failed to retrieve created brief".to_string())
}

#[tauri::command]
pub fn get_brief(app: AppHandle, brief_id: i64) -> Result<Option<AutomationBrief>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_brief(&conn, brief_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_brief_by_cluster(
    app: AppHandle,
    cluster_id: i64,
) -> Result<Option<AutomationBrief>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_brief_by_cluster(&conn, cluster_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_brief(app: AppHandle, brief: AutomationBrief) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_brief(&conn, &brief).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_brief_exported(app: AppHandle, brief_id: i64) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::mark_brief_exported(&conn, brief_id).map_err(|e| e.to_string())
}

// --- Outcome tracking ---

#[tauri::command]
pub fn update_brief_outcome(
    app: AppHandle,
    brief_id: i64,
    status: String,
    estimated_savings_mins: Option<i64>,
    outcome_note: Option<String>,
) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::update_brief_outcome(
        &conn,
        brief_id,
        &status,
        estimated_savings_mins,
        outcome_note.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_briefs_needing_followup(app: AppHandle) -> Result<Vec<AutomationBrief>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_briefs_needing_followup(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_wins(app: AppHandle) -> Result<Vec<WinEntry>, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_wins(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_total_savings_mins(app: AppHandle) -> Result<i64, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::get_total_savings_mins(&conn).map_err(|e| e.to_string())
}

// --- Investigation ---

#[tauri::command]
pub fn generate_investigation_prompt(
    app: AppHandle,
    cluster_id: Option<i64>,
) -> Result<String, String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let clusters = db::get_all_clusters(&conn).map_err(|e| e.to_string())?;
    let logs = db::get_all_logs(&conn).map_err(|e| e.to_string())?;

    // Get current week stats
    let now = chrono::Local::now();
    let week_ago = now - chrono::Duration::days(7);
    let stats = db::get_review_stats(
        &conn,
        &week_ago.format("%Y-%m-%dT%H:%M:%S").to_string(),
        &now.format("%Y-%m-%dT%H:%M:%S").to_string(),
    )
    .ok();

    let prompt =
        investigate::generate_prompt(&clusters, &logs, stats.as_ref(), cluster_id);
    Ok(prompt)
}

#[tauri::command]
pub fn import_investigation_report(
    app: AppHandle,
    report_text: String,
) -> Result<ImportResult, String> {
    let report = investigate::parse_report(&report_text)?;

    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut logs_created: usize = 0;

    for pattern in &report.patterns {
        let category = pattern
            .category
            .as_deref()
            .and_then(investigate::normalize_category);

        let app_context = if pattern.apps.is_empty() {
            None
        } else {
            Some(pattern.apps.join(", "))
        };

        // Build a descriptive note from the pattern
        let mut note = pattern.description.clone();
        if let Some(ref freq) = pattern.frequency {
            note.push_str(&format!(" ({})", freq));
        }
        if let Some(mins) = pattern.estimated_mins_per_occurrence {
            note.push_str(&format!(" ~{}min each", mins));
        }

        db::insert_imported_log(
            &conn,
            &note,
            category,
            Some(0.6), // moderate confidence for imported patterns
            app_context.as_deref(),
            "investigation",
        )
        .map_err(|e| e.to_string())?;
        logs_created += 1;

        // Also create entries for each piece of evidence
        for evidence in &pattern.evidence {
            db::insert_imported_log(
                &conn,
                evidence,
                category,
                Some(0.5),
                app_context.as_deref(),
                "investigation",
            )
            .map_err(|e| e.to_string())?;
            logs_created += 1;
        }
    }

    // Notify frontend to refresh
    let _ = app.emit("log-saved", ());

    Ok(ImportResult {
        patterns_imported: report.patterns.len(),
        logs_created,
    })
}

// --- Data management ---

#[tauri::command]
pub fn delete_all_data(app: AppHandle) -> Result<(), String> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db::delete_all_data(&conn).map_err(|e| e.to_string())
}
