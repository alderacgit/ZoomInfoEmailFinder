// Placeholder: email extractor will collect and filter emails per heuristics

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmailPreference {
    Contact,
    Info,
    Sales,
    Other(String),
}

pub fn rank_local_part(local: &str) -> EmailPreference {
    match local.to_ascii_lowercase().as_str() {
        "contact" => EmailPreference::Contact,
        "info" => EmailPreference::Info,
        "sales" => EmailPreference::Sales,
        other => EmailPreference::Other(other.to_string()),
    }
}
