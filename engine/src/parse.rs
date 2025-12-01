use regex::Regex;
use crate::{Simulation, PackageUpdate, Summary};

// Improved parser: looks for "Inst pkg (ver -> newver)" lines from apt output
pub fn parse_apt_simulation(output: &str) -> Simulation {
    let mut updates = Vec::new();
    let re_inst = Regex::new(r"Inst\s+([^\s]+)\s+\(([^)]+)\)").unwrap();

    for cap in re_inst.captures_iter(output) {
        let name = cap[1].to_string();
        // inside parens, sometimes "current -> new ..."
        let paren = cap[2].to_string();
        let parts: Vec<&str> = paren.split("->").map(|s| s.trim()).collect();
        let (current, new) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("unknown".to_string(), paren.clone())
        };

        // naive risk heuristics:
        // - if package in critical list (openssl, systemd, glibc) => high risk
        // - if major version bump (string) => medium-high risk
        // - else low
        let mut risks = Vec::new();
        let mut score = 0.1_f64;

        let critical = ["openssl", "systemd", "glibc", "libc6", "ld-linux"];
        for c in critical.iter() {
            if name.contains(c) {
                risks.push(format!("{} is critical system package", c));
                score = score.max(0.9);
            }
        }

        // detect large major version bump (very naive)
        if current != "unknown" && new != "unknown" && current.split('.').next() != new.split('.').next() {
            risks.push("major version change detected".into());
            score = score.max(0.7);
        }

        if score < 0.5 {
            // low risk default
            risks.push("recommended: test before deploy".into());
            score = score.max(0.2);
        }

        updates.push(PackageUpdate {
            name,
            current,
            new,
            risks,
            risk_score: Some(score),
        });
    }

    let total = updates.len() as u32;
    let high_risk = updates.iter().filter(|u| u.risk_score.unwrap_or(0.0) >= 0.75).count() as u32;
    let medium_risk = updates.iter().filter(|u| (u.risk_score.unwrap_or(0.0) >= 0.4) && (u.risk_score.unwrap_or(0.0) < 0.75)).count() as u32;

    Simulation { updates, summary: Summary { total, high_risk: high_risk, medium_risk: medium_risk } }
}
