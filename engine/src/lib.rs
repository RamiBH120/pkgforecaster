use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageUpdate {
    pub name: String,
    pub current: String,
    pub new: String,
    pub risks: Vec<String>,
    pub risk_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Simulation {
    pub updates: Vec<PackageUpdate>,
    pub summary: Summary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Summary {
    pub total: u32,
    pub highRisk: u32,
    pub mediumRisk: u32,
}

/// Parse apt simulation text and produce structured Simulation info.
/// For the example we implement a tiny parser that looks for candidate packages.
/// In a real implementation you should call 'apt-get -s dist-upgrade' or use
/// libapt-pkg bindings to get structured results.
// pub fn parse_apt_simulation(output: &str) -> Simulation {
//     // VERY simple stub parser: look for lines like "Inst packagename (version -> newversion)"
//     let mut updates = Vec::new();
//     for line in output.lines() {
//         if line.starts_with("Inst ") {
//             // naive parse
//             // Inst package [arch] (2.0.1-1 Ubuntu:20.04 [amd64]) ...
//             let parts: Vec<&str> = line.split_whitespace().collect();
//             if parts.len() >= 2 {
//                 let name = parts[1].to_string();
//                 updates.push(PackageUpdate {
//                     name,
//                     current: "unknown".into(),
//                     new: "unknown".into(),
//                     risks: vec!["unknown risk (stub)".into()],
//                 });
//             }
//         }
//     }
//     let total = updates.len() as u32;
//     let summary = Summary {
//         total,
//         highRisk: 0,
//         mediumRisk: total,
//     };
//     Simulation { updates, summary }
// }

impl Simulation {
    pub fn new() -> Self {
        Self { updates: vec![], summary: Summary { total: 0, highRisk: 0, mediumRisk: 0 } }
    }
}