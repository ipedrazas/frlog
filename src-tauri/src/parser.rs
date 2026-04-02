use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DotCommand {
    Waiting,      // .w
    Blocking,     // .b
    Repetitive,   // .r
    Coordination, // .c
    GuiltPile,    // .g
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedInput {
    pub command: Option<DotCommand>,
    pub category: Option<String>,
    pub note_text: String,
}

impl DotCommand {
    pub fn category(&self) -> &'static str {
        match self {
            DotCommand::Waiting => "bottleneck",
            DotCommand::Blocking => "bottleneck",
            DotCommand::Repetitive => "repetitive_work",
            DotCommand::Coordination => "coordination_tax",
            DotCommand::GuiltPile => "guilt_pile",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DotCommand::Waiting => "waiting",
            DotCommand::Blocking => "blocking",
            DotCommand::Repetitive => "repetitive",
            DotCommand::Coordination => "coordination",
            DotCommand::GuiltPile => "guilt_pile",
        }
    }
}

pub fn parse(input: &str) -> ParsedInput {
    let trimmed = input.trim();

    let prefixes: &[(&str, DotCommand)] = &[
        (".w ", DotCommand::Waiting),
        (".b ", DotCommand::Blocking),
        (".r ", DotCommand::Repetitive),
        (".c ", DotCommand::Coordination),
        (".g ", DotCommand::GuiltPile),
    ];

    for (prefix, cmd) in prefixes {
        if trimmed.starts_with(prefix) {
            let note_text = trimmed[prefix.len()..].to_string();
            return ParsedInput {
                category: Some(cmd.category().to_string()),
                command: Some(cmd.clone()),
                note_text,
            };
        }
    }

    ParsedInput {
        command: None,
        category: None,
        note_text: trimmed.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_waiting() {
        let p = parse(".w waiting on approval");
        assert_eq!(p.command, Some(DotCommand::Waiting));
        assert_eq!(p.note_text, "waiting on approval");
        assert_eq!(p.category.as_deref(), Some("bottleneck"));
    }

    #[test]
    fn parse_blocking() {
        let p = parse(".b QA needs my logs");
        assert_eq!(p.command, Some(DotCommand::Blocking));
        assert_eq!(p.note_text, "QA needs my logs");
    }

    #[test]
    fn parse_repetitive() {
        let p = parse(".r pasted ticket IDs again");
        assert_eq!(p.command, Some(DotCommand::Repetitive));
        assert_eq!(p.category.as_deref(), Some("repetitive_work"));
    }

    #[test]
    fn parse_coordination() {
        let p = parse(".c chased PM twice");
        assert_eq!(p.command, Some(DotCommand::Coordination));
        assert_eq!(p.category.as_deref(), Some("coordination_tax"));
    }

    #[test]
    fn parse_guilt_pile() {
        let p = parse(".g still haven't cleaned up docs");
        assert_eq!(p.command, Some(DotCommand::GuiltPile));
        assert_eq!(p.category.as_deref(), Some("guilt_pile"));
    }

    #[test]
    fn parse_plain_text() {
        let p = parse("just a normal frustration");
        assert_eq!(p.command, None);
        assert_eq!(p.category, None);
        assert_eq!(p.note_text, "just a normal frustration");
    }

    #[test]
    fn parse_no_space_after_dot() {
        // ".waiting" should not be parsed as a command
        let p = parse(".waiting on something");
        assert_eq!(p.command, None);
    }
}
