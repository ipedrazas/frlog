use crate::db::{AutomationBrief, ClusterDetail, LogEntry};

fn format_category(cat: &str) -> &str {
    match cat {
        "repetitive_work" => "repetitive manual tasks",
        "bottleneck" => "waiting and blocking",
        "coordination_tax" => "coordination overhead",
        "context_switching" => "frequent context switching",
        "guilt_pile" => "consistently deferred tasks",
        _ => cat,
    }
}

pub fn generate(detail: &ClusterDetail) -> AutomationBrief {
    let cluster = &detail.cluster;
    let logs = &detail.logs;
    let apps = &detail.app_contexts;

    let example_texts: Vec<&str> = logs.iter()
        .take(5)
        .map(|l| if l.note_text.is_empty() { l.raw_text.as_str() } else { l.note_text.as_str() })
        .collect();

    let problem = format!(
        "Recurring friction around {}. This pattern has appeared {} times.",
        format_category(&cluster.category),
        cluster.log_count,
    );

    let trigger = if !example_texts.is_empty() {
        format!(
            "This occurs when: {}",
            summarize_trigger(&example_texts)
        )
    } else {
        "Trigger not yet identified from logs.".to_string()
    };

    let current_workflow = example_texts.iter()
        .map(|t| format!("- {}", t))
        .collect::<Vec<_>>()
        .join("\n");

    let apps_involved = if apps.is_empty() {
        "Not identified".to_string()
    } else {
        apps.join(", ")
    };

    let frequency = format!(
        "Approximately {} occurrences logged",
        cluster.log_count,
    );

    let estimated_time_cost = estimate_time_cost(cluster.log_count);

    let emotional_cost = match cluster.category.as_str() {
        "repetitive_work" => "Low complexity, high annoyance — feels like wasted effort",
        "bottleneck" => "Frustrating — progress blocked by external dependency",
        "coordination_tax" => "Draining — energy spent chasing rather than creating",
        "context_switching" => "Fragmenting — hard to maintain deep focus",
        "guilt_pile" => "Nagging — persistent low-grade stress from avoidance",
        _ => "Moderate friction",
    }.to_string();

    let example_instances = example_texts.iter()
        .map(|t| format!("- \"{}\"", t))
        .collect::<Vec<_>>()
        .join("\n");

    let desired_outcome = generate_desired_outcome(&cluster.category, &example_texts);
    let candidate_approaches = generate_approaches(&cluster.category, apps);

    let agent_spec = generate_agent_spec(
        &cluster.title,
        &cluster.category,
        &example_texts,
        apps,
    );

    AutomationBrief {
        id: 0,
        cluster_id: cluster.id,
        problem,
        trigger,
        current_workflow,
        apps_involved,
        frequency,
        estimated_time_cost,
        emotional_cost,
        dependencies: "To be identified based on specific workflow".to_string(),
        example_instances,
        desired_outcome,
        constraints: "- Must be reliable and not introduce new friction\n- Should allow manual review before taking action".to_string(),
        candidate_approaches,
        agent_spec,
        exported_at: None,
        resolution_status: None,
        resolved_at: None,
        estimated_savings_mins: None,
        outcome_note: None,
        created_at: String::new(),
    }
}

fn summarize_trigger(examples: &[&str]) -> String {
    if examples.len() == 1 {
        return examples[0].to_string();
    }
    // Take the first two examples as representative triggers
    examples.iter()
        .take(2)
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; or when ")
}

fn estimate_time_cost(log_count: i64) -> String {
    let est_mins_per = 10;
    let weekly_est = (log_count as f64 * est_mins_per as f64 / 4.0).ceil() as i64;
    if weekly_est < 60 {
        format!("~{} minutes/week (estimated from {} logged occurrences)", weekly_est, log_count)
    } else {
        let hours = weekly_est / 60;
        let mins = weekly_est % 60;
        if mins > 0 {
            format!("~{}h {}m/week (estimated from {} logged occurrences)", hours, mins, log_count)
        } else {
            format!("~{}h/week (estimated from {} logged occurrences)", hours, log_count)
        }
    }
}

fn generate_desired_outcome(category: &str, examples: &[&str]) -> String {
    let base = match category {
        "repetitive_work" => "Eliminate or significantly reduce the manual repetition",
        "bottleneck" => "Remove or shorten the waiting/blocking period",
        "coordination_tax" => "Reduce the number of manual follow-ups and status checks needed",
        "context_switching" => "Consolidate the workflow to reduce unnecessary app switching",
        "guilt_pile" => "Automate or schedule the deferred task so it happens consistently",
        _ => "Reduce the friction in this workflow",
    };
    if !examples.is_empty() {
        format!("{}. Based on logged examples, the ideal solution would handle: {}", base, examples[0])
    } else {
        base.to_string()
    }
}

fn generate_approaches(category: &str, apps: &[String]) -> String {
    let mut approaches = Vec::new();

    match category {
        "repetitive_work" => {
            approaches.push("- Script or automation that performs the repeated steps");
            approaches.push("- Template or shortcut to reduce manual input");
            approaches.push("- AI agent workflow triggered on demand");
        }
        "bottleneck" => {
            approaches.push("- Notification/alert when blocker is resolved");
            approaches.push("- Automated follow-up reminders");
            approaches.push("- Parallel workflow to reduce dependency");
        }
        "coordination_tax" => {
            approaches.push("- Automated status aggregation from source tools");
            approaches.push("- Scheduled digest that replaces manual check-ins");
            approaches.push("- Bot/integration that posts updates automatically");
        }
        "context_switching" => {
            approaches.push("- Unified dashboard aggregating data from multiple tools");
            approaches.push("- Browser extension or integration to reduce tab switching");
            approaches.push("- Keyboard shortcut workflow to streamline transitions");
        }
        "guilt_pile" => {
            approaches.push("- Scheduled automation that runs the task on a cadence");
            approaches.push("- Checklist template with reminders");
            approaches.push("- AI agent that handles the task with minimal input");
        }
        _ => {
            approaches.push("- Custom script or workflow automation");
            approaches.push("- AI agent that handles the repetitive parts");
        }
    }

    if !apps.is_empty() {
        approaches.push(&"- API integration between the involved apps");
    }

    approaches.join("\n")
}

fn generate_agent_spec(title: &str, category: &str, examples: &[&str], apps: &[String]) -> String {
    let task_desc = if !examples.is_empty() {
        examples.iter()
            .take(3)
            .map(|e| format!("  - {}", e))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "  - (describe the specific steps)".to_string()
    };

    let app_list = if apps.is_empty() {
        "(identify the apps/services involved)".to_string()
    } else {
        apps.join(", ")
    };

    format!(
        "I have a recurring {} problem: \"{}\"\n\n\
         Specific examples of this friction:\n{}\n\n\
         Apps/services involved: {}\n\n\
         Build a solution that addresses this pattern. \
         The solution should be reliable, require minimal manual intervention, \
         and allow me to review output before any external action is taken.",
        format_category(category),
        title,
        task_desc,
        app_list,
    )
}
