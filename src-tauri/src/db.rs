use rusqlite::{Connection, OptionalExtension, Result, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

// --- Log entries ---

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub id: i64,
    pub raw_text: String,
    pub note_text: String,
    pub parsed_command: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub app_context: Option<String>,
    pub created_at: String,
}

// --- Waiting periods ---

#[derive(Debug, Serialize)]
pub struct WaitingPeriod {
    pub id: i64,
    pub note: String,
    pub direction: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: Option<i64>,
}

// --- Focus events ---

#[derive(Debug, Serialize)]
pub struct FocusEvent {
    pub id: i64,
    pub app_name: String,
    pub window_title: Option<String>,
    pub created_at: String,
}

// --- Friction clusters ---

#[derive(Debug, Serialize, Clone)]
pub struct FrictionCluster {
    pub id: i64,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub confidence: f64,
    pub status: String,
    pub log_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ClusterDetail {
    pub cluster: FrictionCluster,
    pub logs: Vec<LogEntry>,
    pub app_contexts: Vec<String>,
}

// --- Automation briefs ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationBrief {
    pub id: i64,
    pub cluster_id: i64,
    pub problem: String,
    pub trigger: String,
    pub current_workflow: String,
    pub apps_involved: String,
    pub frequency: String,
    pub estimated_time_cost: String,
    pub emotional_cost: String,
    pub dependencies: String,
    pub example_instances: String,
    pub desired_outcome: String,
    pub constraints: String,
    pub candidate_approaches: String,
    pub agent_spec: String,
    pub exported_at: Option<String>,
    pub resolution_status: Option<String>,
    pub resolved_at: Option<String>,
    pub estimated_savings_mins: Option<i64>,
    pub outcome_note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WinEntry {
    pub brief_id: i64,
    pub cluster_title: String,
    pub category: String,
    pub resolution_status: String,
    pub resolved_at: String,
    pub estimated_savings_mins: Option<i64>,
    pub outcome_note: Option<String>,
    pub exported_at: Option<String>,
}

// --- Review stats ---

#[derive(Debug, Serialize)]
pub struct ReviewStats {
    pub total_logs: i64,
    pub logs_with_category: i64,
    pub total_waiting_secs: i64,
    pub category_counts: Vec<CategoryCount>,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

// --- Privacy rules ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacyRule {
    pub app_name: String,
    pub excluded: bool,
}

// --- Settings ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub tracking_paused: bool,
    pub capture_titles: bool,
    pub onboarding_completed: bool,
    pub poll_interval_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tracking_paused: false,
            capture_titles: false,
            onboarding_completed: false,
            poll_interval_secs: 3,
        }
    }
}

pub fn init(app_data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("frlog.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS manual_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            raw_text TEXT NOT NULL,
            note_text TEXT NOT NULL DEFAULT '',
            parsed_command TEXT,
            category TEXT,
            app_context TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS focus_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL,
            window_title TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS waiting_periods (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            log_id INTEGER,
            note TEXT NOT NULL,
            direction TEXT NOT NULL,
            start_time TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime')),
            end_time TEXT,
            duration_secs INTEGER,
            FOREIGN KEY (log_id) REFERENCES manual_logs(id)
        );

        CREATE TABLE IF NOT EXISTS friction_clusters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.5,
            status TEXT NOT NULL DEFAULT 'new',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS cluster_logs (
            cluster_id INTEGER NOT NULL,
            log_id INTEGER NOT NULL,
            PRIMARY KEY (cluster_id, log_id),
            FOREIGN KEY (cluster_id) REFERENCES friction_clusters(id) ON DELETE CASCADE,
            FOREIGN KEY (log_id) REFERENCES manual_logs(id)
        );

        CREATE TABLE IF NOT EXISTS automation_briefs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cluster_id INTEGER NOT NULL,
            problem TEXT NOT NULL DEFAULT '',
            trigger_desc TEXT NOT NULL DEFAULT '',
            current_workflow TEXT NOT NULL DEFAULT '',
            apps_involved TEXT NOT NULL DEFAULT '',
            frequency TEXT NOT NULL DEFAULT '',
            estimated_time_cost TEXT NOT NULL DEFAULT '',
            emotional_cost TEXT NOT NULL DEFAULT '',
            dependencies TEXT NOT NULL DEFAULT '',
            example_instances TEXT NOT NULL DEFAULT '',
            desired_outcome TEXT NOT NULL DEFAULT '',
            constraints TEXT NOT NULL DEFAULT '',
            candidate_approaches TEXT NOT NULL DEFAULT '',
            agent_spec TEXT NOT NULL DEFAULT '',
            exported_at TEXT,
            resolution_status TEXT,
            resolved_at TEXT,
            estimated_savings_mins INTEGER,
            outcome_note TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime')),
            FOREIGN KEY (cluster_id) REFERENCES friction_clusters(id)
        );

        CREATE TABLE IF NOT EXISTS privacy_rules (
            app_name TEXT PRIMARY KEY,
            excluded INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    )?;

    // Migrations for DBs created in earlier phases
    let has_col = |table: &str, col: &str| -> bool {
        conn.prepare(&format!(
            "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name='{}'", table, col
        ))
        .and_then(|mut s| s.query_row([], |row| row.get::<_, i64>(0)))
        .map(|c| c > 0)
        .unwrap_or(false)
    };

    if !has_col("manual_logs", "app_context") {
        conn.execute_batch("ALTER TABLE manual_logs ADD COLUMN app_context TEXT;")?;
    }
    if !has_col("manual_logs", "note_text") {
        conn.execute_batch("ALTER TABLE manual_logs ADD COLUMN note_text TEXT NOT NULL DEFAULT '';")?;
    }
    if !has_col("manual_logs", "parsed_command") {
        conn.execute_batch("ALTER TABLE manual_logs ADD COLUMN parsed_command TEXT;")?;
    }
    if !has_col("manual_logs", "category") {
        conn.execute_batch("ALTER TABLE manual_logs ADD COLUMN category TEXT;")?;
    }
    if !has_col("manual_logs", "confidence") {
        conn.execute_batch("ALTER TABLE manual_logs ADD COLUMN confidence REAL;")?;
    }
    if !has_col("automation_briefs", "resolved_at") {
        conn.execute_batch("ALTER TABLE automation_briefs ADD COLUMN resolved_at TEXT;")?;
    }
    if !has_col("automation_briefs", "estimated_savings_mins") {
        conn.execute_batch("ALTER TABLE automation_briefs ADD COLUMN estimated_savings_mins INTEGER;")?;
    }
    if !has_col("automation_briefs", "outcome_note") {
        conn.execute_batch("ALTER TABLE automation_briefs ADD COLUMN outcome_note TEXT;")?;
    }

    Ok(conn)
}

// --- Manual logs ---

pub fn insert_log(
    conn: &Connection,
    raw_text: &str,
    note_text: &str,
    parsed_command: Option<&str>,
    category: Option<&str>,
    confidence: Option<f64>,
    app_context: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO manual_logs (raw_text, note_text, parsed_command, category, confidence, app_context) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![raw_text, note_text, parsed_command, category, confidence, app_context],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_all_logs(conn: &Connection) -> Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_text, note_text, parsed_command, category, confidence, app_context, created_at
         FROM manual_logs ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            raw_text: row.get(1)?,
            note_text: row.get(2)?,
            parsed_command: row.get(3)?,
            category: row.get(4)?,
            confidence: row.get(5)?,
            app_context: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_logs_by_category(conn: &Connection, category: &str) -> Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_text, note_text, parsed_command, category, confidence, app_context, created_at
         FROM manual_logs WHERE category = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![category], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            raw_text: row.get(1)?,
            note_text: row.get(2)?,
            parsed_command: row.get(3)?,
            category: row.get(4)?,
            confidence: row.get(5)?,
            app_context: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_logs_by_command(conn: &Connection, command: &str) -> Result<Vec<LogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_text, note_text, parsed_command, category, confidence, app_context, created_at
         FROM manual_logs WHERE parsed_command = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![command], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            raw_text: row.get(1)?,
            note_text: row.get(2)?,
            parsed_command: row.get(3)?,
            category: row.get(4)?,
            confidence: row.get(5)?,
            app_context: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn delete_all_logs(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM manual_logs;
         DELETE FROM focus_events;
         DELETE FROM waiting_periods;"
    )?;
    Ok(())
}

// --- Waiting periods ---

pub fn start_waiting_period(conn: &Connection, log_id: i64, note: &str, direction: &str) -> Result<()> {
    // Close any open period with the same direction
    conn.execute(
        "UPDATE waiting_periods SET
            end_time = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'),
            duration_secs = CAST((julianday('now', 'localtime') - julianday(start_time)) * 86400 AS INTEGER)
         WHERE direction = ?1 AND end_time IS NULL",
        params![direction],
    )?;
    // Start a new one
    conn.execute(
        "INSERT INTO waiting_periods (log_id, note, direction) VALUES (?1, ?2, ?3)",
        params![log_id, note, direction],
    )?;
    Ok(())
}

pub fn end_waiting_period(conn: &Connection, direction: &str) -> Result<bool> {
    let updated = conn.execute(
        "UPDATE waiting_periods SET
            end_time = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'),
            duration_secs = CAST((julianday('now', 'localtime') - julianday(start_time)) * 86400 AS INTEGER)
         WHERE direction = ?1 AND end_time IS NULL",
        params![direction],
    )?;
    Ok(updated > 0)
}

pub fn get_waiting_periods(conn: &Connection) -> Result<Vec<WaitingPeriod>> {
    let mut stmt = conn.prepare(
        "SELECT id, note, direction, start_time, end_time, duration_secs
         FROM waiting_periods ORDER BY start_time DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WaitingPeriod {
            id: row.get(0)?,
            note: row.get(1)?,
            direction: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
            duration_secs: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn get_active_waiting_periods(conn: &Connection) -> Result<Vec<WaitingPeriod>> {
    let mut stmt = conn.prepare(
        "SELECT id, note, direction, start_time, end_time, duration_secs
         FROM waiting_periods WHERE end_time IS NULL ORDER BY start_time DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WaitingPeriod {
            id: row.get(0)?,
            note: row.get(1)?,
            direction: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
            duration_secs: row.get(5)?,
        })
    })?;
    rows.collect()
}

// --- Focus events ---

pub fn insert_focus_event(conn: &Connection, app_name: &str, window_title: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO focus_events (app_name, window_title) VALUES (?1, ?2)",
        params![app_name, window_title],
    )?;
    Ok(())
}

pub fn get_last_focus_event(conn: &Connection) -> Result<Option<(String, Option<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, window_title FROM focus_events ORDER BY id DESC LIMIT 1"
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    match rows.next() {
        Some(Ok(row)) => Ok(Some(row)),
        _ => Ok(None),
    }
}

pub fn get_current_app(conn: &Connection) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT app_name FROM focus_events ORDER BY id DESC LIMIT 1"
    )?;
    let mut rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(Ok(name)) => Ok(Some(name)),
        _ => Ok(None),
    }
}

pub fn get_focus_events(conn: &Connection, limit: i64) -> Result<Vec<FocusEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, app_name, window_title, created_at FROM focus_events ORDER BY created_at DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(FocusEvent {
            id: row.get(0)?,
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn get_focus_event_count(conn: &Connection) -> Result<i64> {
    conn.prepare("SELECT COUNT(*) FROM focus_events")?
        .query_row([], |row| row.get(0))
}

// --- Privacy rules ---

pub fn is_app_excluded(conn: &Connection, app_name: &str) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT excluded FROM privacy_rules WHERE app_name = ?1"
    )?;
    let mut rows = stmt.query_map(params![app_name], |row| row.get::<_, bool>(0))?;
    match rows.next() {
        Some(Ok(excluded)) => Ok(excluded),
        _ => Ok(false),
    }
}

pub fn set_app_excluded(conn: &Connection, app_name: &str, excluded: bool) -> Result<()> {
    conn.execute(
        "INSERT INTO privacy_rules (app_name, excluded) VALUES (?1, ?2)
         ON CONFLICT(app_name) DO UPDATE SET excluded = ?2",
        params![app_name, excluded],
    )?;
    Ok(())
}

pub fn get_excluded_apps(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT app_name FROM privacy_rules WHERE excluded = 1 ORDER BY app_name"
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn remove_exclusion(conn: &Connection, app_name: &str) -> Result<()> {
    conn.execute("DELETE FROM privacy_rules WHERE app_name = ?1", params![app_name])?;
    Ok(())
}

// --- Settings ---

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
    match rows.next() {
        Some(Ok(val)) => Ok(Some(val)),
        _ => Ok(None),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_settings(conn: &Connection) -> Settings {
    let get_bool = |key: &str, default: bool| -> bool {
        get_setting(conn, key)
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(default)
    };
    Settings {
        tracking_paused: get_bool("tracking_paused", false),
        capture_titles: get_bool("capture_titles", false),
        onboarding_completed: get_bool("onboarding_completed", false),
        poll_interval_secs: get_setting(conn, "poll_interval_secs")
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3),
    }
}

// --- Log tag editing ---

pub fn update_log_category(conn: &Connection, log_id: i64, category: &str, confidence: f64) -> Result<()> {
    conn.execute(
        "UPDATE manual_logs SET category = ?1, confidence = ?2 WHERE id = ?3",
        params![category, confidence, log_id],
    )?;
    Ok(())
}

// --- Friction clusters ---

pub fn clear_clusters(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM cluster_logs;
         DELETE FROM friction_clusters;"
    )?;
    Ok(())
}

pub fn insert_cluster(
    conn: &Connection,
    title: &str,
    summary: &str,
    category: &str,
    confidence: f64,
    log_ids: &[i64],
) -> Result<i64> {
    conn.execute(
        "INSERT INTO friction_clusters (title, summary, category, confidence) VALUES (?1, ?2, ?3, ?4)",
        params![title, summary, category, confidence],
    )?;
    let cluster_id = conn.last_insert_rowid();
    for log_id in log_ids {
        conn.execute(
            "INSERT OR IGNORE INTO cluster_logs (cluster_id, log_id) VALUES (?1, ?2)",
            params![cluster_id, log_id],
        )?;
    }
    Ok(cluster_id)
}

pub fn get_all_clusters(conn: &Connection) -> Result<Vec<FrictionCluster>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.summary, c.category, c.confidence, c.status,
                (SELECT COUNT(*) FROM cluster_logs cl WHERE cl.cluster_id = c.id) as log_count,
                c.created_at, c.updated_at
         FROM friction_clusters c
         ORDER BY log_count DESC, c.confidence DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FrictionCluster {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            category: row.get(3)?,
            confidence: row.get(4)?,
            status: row.get(5)?,
            log_count: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn get_cluster_detail(conn: &Connection, cluster_id: i64) -> Result<Option<ClusterDetail>> {
    // Get the cluster
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.summary, c.category, c.confidence, c.status,
                (SELECT COUNT(*) FROM cluster_logs cl WHERE cl.cluster_id = c.id) as log_count,
                c.created_at, c.updated_at
         FROM friction_clusters c WHERE c.id = ?1"
    )?;
    let cluster = stmt.query_row(params![cluster_id], |row| {
        Ok(FrictionCluster {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            category: row.get(3)?,
            confidence: row.get(4)?,
            status: row.get(5)?,
            log_count: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }).optional()?;

    let cluster = match cluster {
        Some(c) => c,
        None => return Ok(None),
    };

    // Get associated logs
    let mut log_stmt = conn.prepare(
        "SELECT m.id, m.raw_text, m.note_text, m.parsed_command, m.category, m.confidence, m.app_context, m.created_at
         FROM manual_logs m
         JOIN cluster_logs cl ON cl.log_id = m.id
         WHERE cl.cluster_id = ?1
         ORDER BY m.created_at DESC"
    )?;
    let logs: Vec<LogEntry> = log_stmt.query_map(params![cluster_id], |row| {
        Ok(LogEntry {
            id: row.get(0)?,
            raw_text: row.get(1)?,
            note_text: row.get(2)?,
            parsed_command: row.get(3)?,
            category: row.get(4)?,
            confidence: row.get(5)?,
            app_context: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    // Get distinct app contexts
    let mut app_stmt = conn.prepare(
        "SELECT DISTINCT m.app_context
         FROM manual_logs m
         JOIN cluster_logs cl ON cl.log_id = m.id
         WHERE cl.cluster_id = ?1 AND m.app_context IS NOT NULL
         ORDER BY m.app_context"
    )?;
    let app_contexts: Vec<String> = app_stmt.query_map(params![cluster_id], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<Vec<_>>>()?;

    Ok(Some(ClusterDetail { cluster, logs, app_contexts }))
}

pub fn update_cluster_status(conn: &Connection, cluster_id: i64, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE friction_clusters SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime') WHERE id = ?2",
        params![status, cluster_id],
    )?;
    Ok(())
}

pub fn update_cluster_title(conn: &Connection, cluster_id: i64, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE friction_clusters SET title = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime') WHERE id = ?2",
        params![title, cluster_id],
    )?;
    Ok(())
}

pub fn update_cluster_category(conn: &Connection, cluster_id: i64, category: &str) -> Result<()> {
    conn.execute(
        "UPDATE friction_clusters SET category = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime') WHERE id = ?2",
        params![category, cluster_id],
    )?;
    Ok(())
}

pub fn get_clusters_for_review(conn: &Connection, exclude_ignored: bool) -> Result<Vec<FrictionCluster>> {
    let sql = if exclude_ignored {
        "SELECT c.id, c.title, c.summary, c.category, c.confidence, c.status,
                (SELECT COUNT(*) FROM cluster_logs cl WHERE cl.cluster_id = c.id) as log_count,
                c.created_at, c.updated_at
         FROM friction_clusters c
         WHERE c.status != 'ignore'
         ORDER BY log_count DESC, c.confidence DESC"
    } else {
        "SELECT c.id, c.title, c.summary, c.category, c.confidence, c.status,
                (SELECT COUNT(*) FROM cluster_logs cl WHERE cl.cluster_id = c.id) as log_count,
                c.created_at, c.updated_at
         FROM friction_clusters c
         ORDER BY log_count DESC, c.confidence DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(FrictionCluster {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            category: row.get(3)?,
            confidence: row.get(4)?,
            status: row.get(5)?,
            log_count: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn get_review_stats(conn: &Connection, since: &str, until: &str) -> Result<ReviewStats> {
    let total_logs: i64 = conn.prepare(
        "SELECT COUNT(*) FROM manual_logs WHERE created_at >= ?1 AND created_at < ?2"
    )?.query_row(params![since, until], |row| row.get(0))?;

    let logs_with_category: i64 = conn.prepare(
        "SELECT COUNT(*) FROM manual_logs WHERE created_at >= ?1 AND created_at < ?2 AND category IS NOT NULL"
    )?.query_row(params![since, until], |row| row.get(0))?;

    let total_waiting_secs: i64 = conn.prepare(
        "SELECT COALESCE(SUM(duration_secs), 0) FROM waiting_periods
         WHERE start_time >= ?1 AND start_time < ?2 AND duration_secs IS NOT NULL"
    )?.query_row(params![since, until], |row| row.get(0))?;

    let mut cat_stmt = conn.prepare(
        "SELECT category, COUNT(*) as cnt FROM manual_logs
         WHERE created_at >= ?1 AND created_at < ?2 AND category IS NOT NULL
         GROUP BY category ORDER BY cnt DESC"
    )?;
    let category_counts: Vec<CategoryCount> = cat_stmt.query_map(params![since, until], |row| {
        Ok(CategoryCount {
            category: row.get(0)?,
            count: row.get(1)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(ReviewStats {
        total_logs,
        logs_with_category,
        total_waiting_secs,
        category_counts,
        period_start: since.to_string(),
        period_end: until.to_string(),
    })
}

// --- Automation briefs ---

pub fn insert_brief(conn: &Connection, brief: &AutomationBrief) -> Result<i64> {
    conn.execute(
        "INSERT INTO automation_briefs (cluster_id, problem, trigger_desc, current_workflow,
         apps_involved, frequency, estimated_time_cost, emotional_cost, dependencies,
         example_instances, desired_outcome, constraints, candidate_approaches, agent_spec)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            brief.cluster_id, brief.problem, brief.trigger, brief.current_workflow,
            brief.apps_involved, brief.frequency, brief.estimated_time_cost,
            brief.emotional_cost, brief.dependencies, brief.example_instances,
            brief.desired_outcome, brief.constraints, brief.candidate_approaches,
            brief.agent_spec,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn row_to_brief(row: &rusqlite::Row) -> rusqlite::Result<AutomationBrief> {
    Ok(AutomationBrief {
        id: row.get(0)?,
        cluster_id: row.get(1)?,
        problem: row.get(2)?,
        trigger: row.get(3)?,
        current_workflow: row.get(4)?,
        apps_involved: row.get(5)?,
        frequency: row.get(6)?,
        estimated_time_cost: row.get(7)?,
        emotional_cost: row.get(8)?,
        dependencies: row.get(9)?,
        example_instances: row.get(10)?,
        desired_outcome: row.get(11)?,
        constraints: row.get(12)?,
        candidate_approaches: row.get(13)?,
        agent_spec: row.get(14)?,
        exported_at: row.get(15)?,
        resolution_status: row.get(16)?,
        resolved_at: row.get(17)?,
        estimated_savings_mins: row.get(18)?,
        outcome_note: row.get(19)?,
        created_at: row.get(20)?,
    })
}

const BRIEF_SELECT: &str =
    "SELECT id, cluster_id, problem, trigger_desc, current_workflow, apps_involved,
     frequency, estimated_time_cost, emotional_cost, dependencies, example_instances,
     desired_outcome, constraints, candidate_approaches, agent_spec,
     exported_at, resolution_status, resolved_at, estimated_savings_mins, outcome_note, created_at
     FROM automation_briefs";

pub fn get_brief(conn: &Connection, brief_id: i64) -> Result<Option<AutomationBrief>> {
    let sql = format!("{} WHERE id = ?1", BRIEF_SELECT);
    conn.prepare(&sql)?.query_row(params![brief_id], row_to_brief).optional()
}

pub fn get_brief_by_cluster(conn: &Connection, cluster_id: i64) -> Result<Option<AutomationBrief>> {
    let sql = format!("{} WHERE cluster_id = ?1 ORDER BY created_at DESC LIMIT 1", BRIEF_SELECT);
    conn.prepare(&sql)?.query_row(params![cluster_id], row_to_brief).optional()
}

pub fn update_brief(conn: &Connection, brief: &AutomationBrief) -> Result<()> {
    conn.execute(
        "UPDATE automation_briefs SET problem = ?1, trigger_desc = ?2, current_workflow = ?3,
         apps_involved = ?4, frequency = ?5, estimated_time_cost = ?6, emotional_cost = ?7,
         dependencies = ?8, example_instances = ?9, desired_outcome = ?10, constraints = ?11,
         candidate_approaches = ?12, agent_spec = ?13
         WHERE id = ?14",
        params![
            brief.problem, brief.trigger, brief.current_workflow,
            brief.apps_involved, brief.frequency, brief.estimated_time_cost,
            brief.emotional_cost, brief.dependencies, brief.example_instances,
            brief.desired_outcome, brief.constraints, brief.candidate_approaches,
            brief.agent_spec, brief.id,
        ],
    )?;
    Ok(())
}

pub fn mark_brief_exported(conn: &Connection, brief_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE automation_briefs SET exported_at = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime') WHERE id = ?1",
        params![brief_id],
    )?;
    Ok(())
}

pub fn update_brief_outcome(
    conn: &Connection,
    brief_id: i64,
    status: &str,
    estimated_savings_mins: Option<i64>,
    outcome_note: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE automation_briefs SET
            resolution_status = ?1,
            resolved_at = strftime('%Y-%m-%dT%H:%M:%f', 'now', 'localtime'),
            estimated_savings_mins = ?2,
            outcome_note = ?3
         WHERE id = ?4",
        params![status, estimated_savings_mins, outcome_note, brief_id],
    )?;
    Ok(())
}

pub fn get_briefs_needing_followup(conn: &Connection) -> Result<Vec<AutomationBrief>> {
    let sql = format!(
        "{} WHERE exported_at IS NOT NULL AND resolution_status IS NULL
         AND julianday('now', 'localtime') - julianday(exported_at) >= 7
         ORDER BY exported_at ASC",
        BRIEF_SELECT
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_brief)?;
    rows.collect()
}

pub fn get_wins(conn: &Connection) -> Result<Vec<WinEntry>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, c.title, c.category, b.resolution_status, b.resolved_at,
                b.estimated_savings_mins, b.outcome_note, b.exported_at
         FROM automation_briefs b
         JOIN friction_clusters c ON c.id = b.cluster_id
         WHERE b.resolution_status IS NOT NULL
         ORDER BY b.resolved_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(WinEntry {
            brief_id: row.get(0)?,
            cluster_title: row.get(1)?,
            category: row.get(2)?,
            resolution_status: row.get(3)?,
            resolved_at: row.get(4)?,
            estimated_savings_mins: row.get(5)?,
            outcome_note: row.get(6)?,
            exported_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_total_savings_mins(conn: &Connection) -> Result<i64> {
    conn.prepare(
        "SELECT COALESCE(SUM(estimated_savings_mins), 0) FROM automation_briefs
         WHERE resolution_status IN ('resolved', 'reduced') AND estimated_savings_mins IS NOT NULL"
    )?.query_row([], |row| row.get(0))
}

pub fn get_logs_for_clustering(conn: &Connection) -> Result<Vec<(i64, String, Option<String>, Option<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_text, category, app_context FROM manual_logs ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    rows.collect()
}
