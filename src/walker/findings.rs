// Store findings and do whatever with them

use ethers_solc::artifacts::ast::SourceLocation;
use std::collections::{BTreeMap, HashMap};
use yansi::Paint;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Severity {
    Informal,
    Gas,
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn format(&self, text: String) -> String {
        match *self {
            Severity::Informal => format!("{}", Paint::blue(text)),
            Severity::Gas => format!("{}", Paint::magenta(text)),
            Severity::Low => format!("{}", Paint::green(text)),
            Severity::Medium => format!("{}", Paint::yellow(text)),
            Severity::High => format!("{}", Paint::red(text)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub src: Option<SourceLocation>, // Option<SourceLocation>,
    pub code: u32,                   // Identify finding type easily
                                     // pub likelyhood: u8,              // 0-100% likelyhood to be correct
}

#[derive(Debug, Clone)]
pub struct FindingKey {
    pub description: String,
    pub severity: Severity,
}

pub type FindingMap = BTreeMap<u32, FindingKey>;

impl Finding {
    pub fn format(&self) -> String {
        self.severity.format(format!(
            "{}: {}",
            self.name.clone(),
            self.description.clone()
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub file: String,
    pub line: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MetaFinding {
    pub finding: Finding,
    pub meta: Meta,
}

impl MetaFinding {
    pub fn format(&self) -> String {
        self.finding.severity.format(format!(
            "{} {} {}",
            self.meta.file.clone(),
            self.meta
                .line
                .map(|line| format!("l{line}"))
                .unwrap_or_default(),
            self.finding.format()
        ))
    }
}

pub type Findings = Vec<MetaFinding>;

/// Module name -> Findings
pub type AllFindings = HashMap<String, Findings>;

// impl AllFindings {
//     fn more_likelyhood(&self, value: u8) -> AllFindings {
//         &self.iter().flat_map(|(_, finding)| {
//             finding.iter().filter_map(|mf| {
//                 let finding = mf.finding;
//                 Some(finding.likelyhood >= value)
//             })
//         })
//     }
// }
