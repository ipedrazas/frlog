use crate::db::{FrictionCluster, LogEntry, ReviewStats};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize a value that should be a number but might be a string or anything else.
/// Returns None for non-numeric values instead of failing.
fn lenient_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => Ok(n.as_f64()),
        serde_json::Value::String(s) => Ok(s.parse::<f64>().ok()),
        serde_json::Value::Null => Ok(None),
        _ => Ok(None),
    }
}

fn lenient_i64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => Ok(n.as_i64()),
        serde_json::Value::String(s) => Ok(s.parse::<i64>().ok()),
        serde_json::Value::Null => Ok(None),
        _ => Ok(None),
    }
}

/// The schema that external AI agents should return.
/// All fields use lenient parsing — unexpected types are silently ignored
/// rather than failing the entire import.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvestigationReport {
    #[serde(default)]
    pub patterns: Vec<DiscoveredPattern>,
    #[serde(default)]
    pub time_summary: Option<TimeSummary>,
    // Accept and ignore any extra top-level fields (e.g. "meta")
    #[serde(flatten)]
    _extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveredPattern {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub estimated_mins_per_occurrence: Option<i64>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub apps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSummary {
    #[serde(default, deserialize_with = "lenient_f64")]
    pub meetings_hours_per_week: Option<f64>,
    #[serde(default, deserialize_with = "lenient_f64")]
    pub email_hours_per_week: Option<f64>,
    #[serde(default)]
    pub top_coordination_contacts: Vec<String>,
    // Accept and ignore extra fields (e.g. "meetings_breakdown", "busiest_days")
    #[serde(flatten)]
    _extra: HashMap<String, serde_json::Value>,
}

/// Result of importing a report.
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub patterns_imported: usize,
    pub logs_created: usize,
}

fn format_category(cat: &str) -> &str {
    match cat {
        "repetitive_work" => "repetitive tasks",
        "bottleneck" => "waiting and blocking",
        "coordination_tax" => "coordination overhead",
        "context_switching" => "context switching",
        "guilt_pile" => "consistently deferred tasks",
        _ => cat,
    }
}

const REPORT_SCHEMA: &str = r#"{
  "patterns": [
    {
      "description": "Short description of the pattern found",
      "category": "one of: repetitive_work, bottleneck, coordination_tax, context_switching, guilt_pile",
      "frequency": "e.g. daily, 3x/week, weekly",
      "estimated_mins_per_occurrence": 15,
      "evidence": ["specific examples from the data you analyzed"],
      "apps": ["app or service names involved"]
    }
  ],
  "time_summary": {
    "meetings_hours_per_week": 12.5,
    "email_hours_per_week": 6,
    "top_coordination_contacts": ["Person or team you coordinate with most"]
  }
}"#;

/// Generate a context-aware investigation prompt from the user's friction data.
pub fn generate_prompt(
    clusters: &[FrictionCluster],
    recent_logs: &[LogEntry],
    stats: Option<&ReviewStats>,
    cluster_id: Option<i64>,
) -> String {
    let mut prompt = String::new();

    // Opening context
    prompt.push_str(
        "I use a friction logging tool to track moments of frustration, \
         repetition, waiting, and inefficiency in my work. \
         I need your help analyzing my email and calendar to find patterns \
         I might be missing and to validate what I've already noticed.\n\n",
    );

    // What the user has already noticed
    if let Some(cid) = cluster_id {
        // Focused on a specific cluster
        if let Some(cluster) = clusters.iter().find(|c| c.id == cid) {
            prompt.push_str(&format!(
                "## Focus area\n\n\
                 I've identified a specific friction pattern: \"{}\"\n\
                 Category: {}\n\
                 I've logged this {} times.\n\n\
                 Please look specifically for evidence of this pattern in my email/calendar, \
                 and find related patterns I might not have noticed.\n\n",
                cluster.title,
                format_category(&cluster.category),
                cluster.log_count,
            ));
        }
    }

    // Existing patterns summary
    let active_clusters: Vec<_> = clusters
        .iter()
        .filter(|c| c.status != "ignore")
        .take(5)
        .collect();
    if !active_clusters.is_empty() {
        prompt.push_str("## What I've already noticed\n\n");
        for c in &active_clusters {
            prompt.push_str(&format!(
                "- **{}** ({}, {} occurrences)\n",
                c.title,
                format_category(&c.category),
                c.log_count,
            ));
        }
        prompt.push('\n');
    }

    // Recent log examples for context
    let examples: Vec<_> = recent_logs
        .iter()
        .filter(|l| !l.note_text.is_empty())
        .take(8)
        .collect();
    if !examples.is_empty() {
        prompt.push_str("## Recent friction examples\n\n");
        for log in &examples {
            let cat = log
                .category
                .as_deref()
                .map(|c| format!(" [{}]", format_category(c)))
                .unwrap_or_default();
            prompt.push_str(&format!("- \"{}\"{}\n", log.note_text, cat));
        }
        prompt.push('\n');
    }

    // Stats context
    if let Some(s) = stats {
        prompt.push_str(&format!(
            "## This week's stats\n\n\
             - {} friction logs recorded\n\
             - {} categorized\n",
            s.total_logs, s.logs_with_category,
        ));
        if s.total_waiting_secs > 0 {
            let mins = s.total_waiting_secs / 60;
            prompt.push_str(&format!("- {} minutes in waiting/blocked states\n", mins));
        }
        if !s.category_counts.is_empty() {
            prompt.push_str("- Top categories: ");
            let cats: Vec<String> = s
                .category_counts
                .iter()
                .take(3)
                .map(|c| format!("{} ({})", format_category(&c.category), c.count))
                .collect();
            prompt.push_str(&cats.join(", "));
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // The ask
    prompt.push_str(
        "## What I need you to do\n\n\
         Analyze my email and calendar for the last 2-4 weeks. Look for:\n\n\
         1. **Recurring patterns** — repeated tasks, follow-up loops, status update chains\n\
         2. **Coordination overhead** — threads where I'm chasing updates or approvals\n\
         3. **Context switching** — rapid back-and-forth between different projects/topics\n\
         4. **Time sinks** — meetings that could be async, email threads that drag on\n\
         5. **Patterns I haven't noticed** — anything that looks like friction I didn't manually log\n\n",
    );

    // Output format
    prompt.push_str(&format!(
        "## Required output format\n\n\
         Return your findings as JSON matching this exact schema:\n\n\
         ```json\n{}\n```\n\n\
         Important:\n\
         - Use the exact category values listed in the schema\n\
         - Include specific evidence from my email/calendar, not generic observations\n\
         - Estimate time per occurrence based on what you can see\n\
         - List the apps/services involved (Gmail, Calendar, Slack, Jira, etc.)\n\
         - The time_summary section helps me understand my overall time allocation\n\n\
         Return ONLY the JSON — no commentary before or after it.\n",
        REPORT_SCHEMA,
    ));

    prompt
}

/// Parse a JSON investigation report from text.
/// Handles the case where the JSON is wrapped in markdown code fences.
pub fn parse_report(text: &str) -> Result<InvestigationReport, String> {
    let trimmed = text.trim();

    // Strip markdown code fences if present
    let json_str = if trimmed.starts_with("```") {
        let without_start = if let Some(rest) = trimmed.strip_prefix("```json") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("```") {
            rest
        } else {
            trimmed
        };
        without_start
            .strip_suffix("```")
            .unwrap_or(without_start)
            .trim()
    } else {
        trimmed
    };

    serde_json::from_str(json_str).map_err(|e| format!("Failed to parse report JSON: {}", e))
}

/// Map a category string from the report to our internal category values.
/// Accepts various formats and normalizes them.
pub fn normalize_category(cat: &str) -> Option<&'static str> {
    match cat.to_lowercase().replace(['-', ' '], "_").as_str() {
        "repetitive_work" | "repetitive" => Some("repetitive_work"),
        "bottleneck" | "waiting" | "blocking" => Some("bottleneck"),
        "coordination_tax" | "coordination" => Some("coordination_tax"),
        "context_switching" | "context_switch" => Some("context_switching"),
        "guilt_pile" | "guilt" | "deferred" => Some("guilt_pile"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_report() {
        let json = r#"{
            "patterns": [
                {
                    "description": "Weekly status email to 3 stakeholders",
                    "category": "repetitive_work",
                    "frequency": "weekly",
                    "estimated_mins_per_occurrence": 25,
                    "evidence": ["Found 4 similar email threads"],
                    "apps": ["Gmail", "Google Docs"]
                }
            ],
            "time_summary": {
                "meetings_hours_per_week": 12.5,
                "email_hours_per_week": 6,
                "top_coordination_contacts": ["Alice", "Bob"]
            }
        }"#;
        let report = parse_report(json).unwrap();
        assert_eq!(report.patterns.len(), 1);
        assert_eq!(report.patterns[0].description, "Weekly status email to 3 stakeholders");
        assert_eq!(report.patterns[0].category.as_deref(), Some("repetitive_work"));
        assert_eq!(report.time_summary.unwrap().top_coordination_contacts.len(), 2);
    }

    #[test]
    fn parse_report_with_code_fences() {
        let json = "```json\n{\"patterns\": [], \"time_summary\": null}\n```";
        let report = parse_report(json).unwrap();
        assert!(report.patterns.is_empty());
    }

    #[test]
    fn parse_report_minimal() {
        let json = r#"{"patterns": []}"#;
        let report = parse_report(json).unwrap();
        assert!(report.patterns.is_empty());
        assert!(report.time_summary.is_none());
    }

    #[test]
    fn parse_report_invalid() {
        let result = parse_report("not json");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_categories() {
        assert_eq!(normalize_category("repetitive_work"), Some("repetitive_work"));
        assert_eq!(normalize_category("Repetitive Work"), Some("repetitive_work"));
        assert_eq!(normalize_category("coordination"), Some("coordination_tax"));
        assert_eq!(normalize_category("context-switching"), Some("context_switching"));
        assert_eq!(normalize_category("unknown_thing"), None);
    }

    #[test]
    fn parse_report_with_string_numbers() {
        // LLMs sometimes return strings where numbers are expected
        let json = r#"{
            "patterns": [{
                "description": "Test pattern",
                "estimated_mins_per_occurrence": "about 15"
            }],
            "time_summary": {
                "meetings_hours_per_week": 4.5,
                "email_hours_per_week": "unknown — Gmail not connected",
                "top_coordination_contacts": ["Alice"]
            }
        }"#;
        let report = parse_report(json).unwrap();
        assert_eq!(report.patterns.len(), 1);
        // "about 15" can't parse to i64, so it becomes None
        assert_eq!(report.patterns[0].estimated_mins_per_occurrence, None);
        let ts = report.time_summary.unwrap();
        assert_eq!(ts.meetings_hours_per_week, Some(4.5));
        // "unknown — Gmail not connected" becomes None, not an error
        assert_eq!(ts.email_hours_per_week, None);
    }

    #[test]
    fn parse_report_with_extra_fields() {
        // LLMs often add extra fields like "meta", "meetings_breakdown", etc.
        let json = r#"{
            "meta": {
                "data_sources": ["Google Calendar"],
                "period_analyzed": "2026-03-05 to 2026-04-02"
            },
            "patterns": [{
                "description": "Test",
                "category": "bottleneck"
            }],
            "time_summary": {
                "meetings_hours_per_week": 4.5,
                "meetings_breakdown": {"recurring": 2, "ad_hoc": 2.5},
                "email_hours_per_week": null,
                "top_coordination_contacts": [],
                "busiest_days": ["Thursday"]
            }
        }"#;
        let report = parse_report(json).unwrap();
        assert_eq!(report.patterns.len(), 1);
        assert_eq!(report.time_summary.unwrap().meetings_hours_per_week, Some(4.5));
    }
}
