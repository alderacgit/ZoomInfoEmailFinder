use regex::Regex;
use std::collections::HashSet;

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

pub fn normalize_email(e: &str) -> String {
    e.trim().to_ascii_lowercase()
}

pub fn validate_email(e: &str) -> bool {
    email_address::EmailAddress::is_valid(e)
}

pub fn extract_emails_from_html(html: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    // Obvious mailto links
    let mailto_re = Regex::new(r#"mailto:([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})"#).unwrap();
    for cap in mailto_re.captures_iter(html) {
        let e = normalize_email(&cap[1]);
        if validate_email(&e) { set.insert(e); }
    }
    // Raw emails
    let email_re = Regex::new(r"(?i)(?:^|[^a-z0-9._%+-])([a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,})").unwrap();
    for cap in email_re.captures_iter(html) {
        let e = normalize_email(&cap[1]);
        if validate_email(&e) { set.insert(e); }
    }
    set
}

pub fn choose_best_email<'a>(candidates: impl IntoIterator<Item = &'a String>, website_host: &str) -> Option<String> {
    let mut best_same: Option<String> = None;
    let mut best_other: Option<String> = None;

    for e in candidates {
        let parts: Vec<&str> = e.split('@').collect();
        if parts.len() != 2 { continue; }
        let local = parts[0];
        let domain = parts[1];
        let pref = rank_local_part(local);
        let is_same = is_same_domain(domain, website_host);

        let slot = if is_same { &mut best_same } else { &mut best_other };
        match slot {
            Some(current) => {
                if better(e, current, &pref) { *slot = Some(e.clone()); }
            }
            None => *slot = Some(e.clone()),
        }
    }

    best_same.or(best_other)
}

fn better(candidate: &str, current: &str, cand_pref: &EmailPreference) -> bool {
    // Compare local-part preference first
    let cur_local = current.split('@').next().unwrap_or("");
    let cur_pref = rank_local_part(cur_local);
    if cand_pref < &cur_pref { return true; }
    if cand_pref > &cur_pref { return false; }
    // Tiebreak lexicographically
    candidate < current
}

fn is_same_domain(email_domain: &str, website_host: &str) -> bool {
    let email_root = root_host(email_domain);
    let site_root = root_host(website_host);
    email_root.eq_ignore_ascii_case(&site_root)
}

fn root_host(host: &str) -> String {
    let h = host.trim().trim_start_matches("www.");
    h.to_string()
}

#[cfg(test)]
mod tests {
    use super::rank_local_part;

    #[test]
    fn ranks_contact_info_sales() {
        assert!(rank_local_part("contact") < rank_local_part("info"));
        assert!(rank_local_part("info") < rank_local_part("sales"));
        assert!(rank_local_part("sales") < rank_local_part("zach"));
    }
}
