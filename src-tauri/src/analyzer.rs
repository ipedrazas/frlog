use std::collections::{HashMap, HashSet};

/// Auto-infer a friction category from plain text (when no dot command was used).
/// Returns (category, confidence) or None.
pub fn infer_category(text: &str) -> Option<(&'static str, f64)> {
    let lower = text.to_lowercase();

    let rules: &[(&[&str], &str, f64)] = &[
        // Repetitive work
        (
            &[
                "again",
                "every time",
                "same thing",
                "repeat",
                "once more",
                "as usual",
                "copy-past",
                "copy past",
                "re-doing",
                "redoing",
                "rewriting",
                "yet again",
                "for the nth time",
                "groundhog",
                "manually",
            ],
            "repetitive_work",
            0.7,
        ),
        // Bottleneck / waiting
        (
            &[
                "waiting",
                "blocked",
                "pending",
                "approval",
                "stuck",
                "can't proceed",
                "hold up",
                "holdup",
                "sign-off",
                "signoff",
                "depends on",
                "dependency",
                "stalled",
            ],
            "bottleneck",
            0.7,
        ),
        // Coordination tax
        (
            &[
                "chasing",
                "follow up",
                "follow-up",
                "followup",
                "asked twice",
                "ping",
                "slack",
                "meeting",
                "sync",
                "status update",
                "checking in",
                "nudge",
                "reminded",
                "where is",
                "any update",
            ],
            "coordination_tax",
            0.7,
        ),
        // Context switching
        (
            &[
                "switching",
                "back and forth",
                "context switch",
                "interrupted",
                "distracted",
                "lost focus",
                "alt-tab",
                "juggling",
                "bouncing between",
            ],
            "context_switching",
            0.65,
        ),
        // Guilt pile
        (
            &[
                "should do",
                "never get to",
                "keep postponing",
                "putting off",
                "procrastinat",
                "overdue",
                "haven't done",
                "keep meaning to",
                "guilty",
                "neglect",
                "avoiding",
            ],
            "guilt_pile",
            0.65,
        ),
    ];

    let mut best: Option<(&str, f64)> = None;

    for (keywords, category, base_confidence) in rules {
        let mut matches = 0;
        for kw in *keywords {
            if lower.contains(kw) {
                matches += 1;
            }
        }
        if matches > 0 {
            // Boost confidence slightly for multiple keyword matches
            let confidence = (base_confidence + 0.1 * (matches - 1) as f64).min(0.95);
            if best.is_none() || confidence > best.unwrap().1 {
                best = Some((category, confidence));
            }
        }
    }

    best
}

/// Represents a computed cluster of similar logs.
#[derive(Debug, Clone)]
pub struct ComputedCluster {
    pub title: String,
    pub summary: String,
    pub category: String,
    pub confidence: f64,
    pub log_ids: Vec<i64>,
    pub app_contexts: Vec<String>,
}

/// Simple log representation for clustering.
pub struct LogForClustering {
    pub id: i64,
    pub note_text: String,
    pub category: Option<String>,
    pub app_context: Option<String>,
}

/// Cluster logs by category and keyword similarity.
pub fn compute_clusters(logs: &[LogForClustering]) -> Vec<ComputedCluster> {
    // Group logs by category first
    let mut by_category: HashMap<String, Vec<&LogForClustering>> = HashMap::new();
    for log in logs {
        if let Some(ref cat) = log.category {
            by_category.entry(cat.clone()).or_default().push(log);
        }
    }

    let mut clusters = Vec::new();

    for (category, cat_logs) in &by_category {
        if cat_logs.len() < 2 {
            // Single log in a category — still create a cluster for visibility
            let log = cat_logs[0];
            clusters.push(ComputedCluster {
                title: truncate(&log.note_text, 60),
                summary: log.note_text.clone(),
                category: category.clone(),
                confidence: 0.4,
                log_ids: vec![log.id],
                app_contexts: log.app_context.iter().cloned().collect(),
            });
            continue;
        }

        // Sub-cluster within category by keyword similarity
        let sub = sub_cluster(cat_logs, category);
        clusters.extend(sub);
    }

    // Sort by number of logs descending
    clusters.sort_by(|a, b| b.log_ids.len().cmp(&a.log_ids.len()));
    clusters
}

fn sub_cluster(logs: &[&LogForClustering], category: &str) -> Vec<ComputedCluster> {
    // Extract word sets for each log
    let word_sets: Vec<HashSet<String>> =
        logs.iter().map(|l| extract_words(&l.note_text)).collect();

    let n = logs.len();
    let mut assigned = vec![false; n];
    let mut clusters = Vec::new();

    for i in 0..n {
        if assigned[i] {
            continue;
        }
        assigned[i] = true;

        let mut group_ids = vec![logs[i].id];
        let mut group_words = word_sets[i].clone();
        let mut group_apps: HashSet<String> = HashSet::new();
        if let Some(ref app) = logs[i].app_context {
            group_apps.insert(app.clone());
        }

        // Find similar logs
        for j in (i + 1)..n {
            if assigned[j] {
                continue;
            }
            let similarity = jaccard(&word_sets[i], &word_sets[j]);
            // Also consider app context match
            let app_match = match (&logs[i].app_context, &logs[j].app_context) {
                (Some(a), Some(b)) if a == b => 0.15,
                _ => 0.0,
            };
            if similarity + app_match >= 0.25 {
                assigned[j] = true;
                group_ids.push(logs[j].id);
                group_words.extend(word_sets[j].iter().cloned());
                if let Some(ref app) = logs[j].app_context {
                    group_apps.insert(app.clone());
                }
            }
        }

        let count = group_ids.len();
        let confidence = if count >= 5 {
            0.9
        } else if count >= 3 {
            0.75
        } else if count >= 2 {
            0.6
        } else {
            0.4
        };

        // Generate title from most common meaningful words
        let title = generate_cluster_title(
            &group_ids
                .iter()
                .filter_map(|id| logs.iter().find(|l| l.id == *id))
                .map(|l| l.note_text.as_str())
                .collect::<Vec<_>>(),
            category,
        );

        let mut app_list: Vec<String> = group_apps.into_iter().collect();
        app_list.sort();

        let summary = format!(
            "{} logs about {}. {}",
            count,
            format_category(category),
            if !app_list.is_empty() {
                format!("Apps: {}", app_list.join(", "))
            } else {
                String::new()
            }
        );

        clusters.push(ComputedCluster {
            title,
            summary,
            category: category.to_string(),
            confidence,
            log_ids: group_ids,
            app_contexts: app_list,
        });
    }

    clusters
}

fn extract_words(text: &str) -> HashSet<String> {
    let stop_words: HashSet<&str> = [
        "a", "an", "the", "is", "was", "are", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "about", "up",
        "out", "it", "its", "i", "my", "me", "we", "our", "you", "your", "he", "she", "they",
        "them", "this", "that", "these", "those", "and", "or", "but", "not", "no", "so", "if",
        "then", "just", "also", "too", "very",
    ]
    .into_iter()
    .collect();

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(|w| w.to_string())
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    intersection / union
}

fn generate_cluster_title(texts: &[&str], category: &str) -> String {
    // Count word frequency across all texts
    let mut freq: HashMap<String, usize> = HashMap::new();
    for text in texts {
        let words = extract_words(text);
        for w in words {
            *freq.entry(w).or_default() += 1;
        }
    }

    // Pick top 3 most frequent words
    let mut sorted: Vec<_> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let top_words: Vec<String> = sorted.into_iter().take(3).map(|(w, _)| w).collect();

    if top_words.is_empty() {
        format_category(category).to_string()
    } else {
        format!("{}: {}", format_category(category), top_words.join(", "))
    }
}

fn format_category(cat: &str) -> &str {
    match cat {
        "repetitive_work" => "Repetitive work",
        "bottleneck" => "Bottleneck",
        "coordination_tax" => "Coordination tax",
        "context_switching" => "Context switching",
        "guilt_pile" => "Guilt pile",
        _ => cat,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_repetitive() {
        let r = infer_category("pasted the same ticket IDs again");
        assert!(r.is_some());
        let (cat, conf) = r.unwrap();
        assert_eq!(cat, "repetitive_work");
        assert!(conf >= 0.7);
    }

    #[test]
    fn infer_bottleneck() {
        let r = infer_category("still waiting on legal approval");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, "bottleneck");
    }

    #[test]
    fn infer_coordination() {
        let r = infer_category("had to follow up with PM twice about the spec");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, "coordination_tax");
    }

    #[test]
    fn infer_context_switching() {
        let r = infer_category("keep switching back and forth between Jira and Slack");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, "context_switching");
    }

    #[test]
    fn infer_guilt() {
        let r = infer_category("keep postponing the weekly cleanup");
        assert!(r.is_some());
        assert_eq!(r.unwrap().0, "guilt_pile");
    }

    #[test]
    fn infer_none() {
        let r = infer_category("had a good lunch today");
        assert!(r.is_none());
    }

    #[test]
    fn infer_multi_keyword_boosts_confidence() {
        let r = infer_category("copy-pasting the same thing again and again");
        assert!(r.is_some());
        let (_, conf) = r.unwrap();
        assert!(conf > 0.7, "multiple keywords should boost confidence");
    }

    #[test]
    fn clustering_groups_similar() {
        let logs = vec![
            LogForClustering {
                id: 1,
                note_text: "pasted ticket IDs again".into(),
                category: Some("repetitive_work".into()),
                app_context: Some("Jira".into()),
            },
            LogForClustering {
                id: 2,
                note_text: "pasted ticket IDs into Slack again".into(),
                category: Some("repetitive_work".into()),
                app_context: Some("Jira".into()),
            },
            LogForClustering {
                id: 3,
                note_text: "waiting on approval".into(),
                category: Some("bottleneck".into()),
                app_context: None,
            },
        ];
        let clusters = compute_clusters(&logs);
        // The two repetitive logs should cluster together
        let rep_cluster = clusters
            .iter()
            .find(|c| c.category == "repetitive_work")
            .unwrap();
        assert_eq!(rep_cluster.log_ids.len(), 2);
        assert!(rep_cluster.log_ids.contains(&1));
        assert!(rep_cluster.log_ids.contains(&2));
    }
}
