// Store findings and do whatever with them

use ethers_solc::artifacts::ast::SourceLocation;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};
use yansi::Paint;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
            Severity::Gas => write!(f, "gas"),
            Severity::Informal => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub severity: Severity,
    pub src: Option<SourceLocation>, // Option<SourceLocation>,
    pub code: usize,                 // Identify finding type easily
                                     // pub likelyhood: u8,              // 0-100% likelyhood to be correct
}

#[derive(Debug, Clone)]
pub struct FindingKey {
    pub description: String,
    pub summary: String,
    pub severity: Severity,
}

pub type FindingMap = BTreeMap<usize, FindingKey>;

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
    pub line: Option<usize>,
    pub position: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MetaFinding {
    pub finding: Finding,
    pub meta: Meta,
}

impl MetaFinding {
    pub fn format(&self) -> String {
        self.finding.severity.format(format!(
            "{} {}:{} {}",
            self.meta.file.clone(),
            self.meta
                .line
                .map(|line| format!("l{line}"))
                .unwrap_or_default(),
            self.meta.position.unwrap_or_default(),
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

#[derive(Debug, Default)]
pub struct Inside {
    pub function: bool,
    pub unchecked: bool,
    pub for_loop: bool,
    pub while_loop: bool,
}
