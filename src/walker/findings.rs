// Store findings and do whatever with them

use ethers_solc::artifacts::ast::SourceLocation;
use itertools::Itertools;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    path::PathBuf,
};
use yansi::Paint;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Severity {
    Informal,
    Gas,
    Low,
    Medium,
    High,
}

impl Severity {
    #[allow(unused)]
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

impl From<Severity> for u16 {
    fn from(val: Severity) -> Self {
        match val {
            Severity::Informal => 0,
            Severity::Gas => 1,
            Severity::Low => 2,
            Severity::Medium => 3,
            Severity::High => 4,
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Self::Gas
    }
}

#[derive(Default, Clone)]
pub struct ModuleState {
    /// name of the visitor
    pub name: String,
    /// findings included in the visitor pushed during the walking
    pub findings: Vec<Finding>,
    /// dynamic absolute path of the current file containing the artifact being visited
    pub current_file: PathBuf,
    pub file_findings: HashMap<String, Vec<Finding>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct Finding {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub severity: Severity,
    pub src: Option<SourceLocation>, // Option<SourceLocation>,
    pub code: usize,                 // Identify finding type easily
    // pub likelyhood: u8,              // 0-100% likelyhood to be correct
    /// additional comments
    pub comment: Option<String>,
    /// gas saved
    pub gas: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FindingKey {
    pub summary: String,
    pub description: String,
    pub severity: Severity,
}

pub type FindingMap = BTreeMap<usize, FindingKey>;

impl Finding {
    #[allow(unused)]
    pub fn format(&self) -> String {
        self.severity.format(format!(
            "{}: {}",
            self.name.clone(),
            self.description.clone()
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Meta {
    /// Path of the file relative to the project path
    pub file: String,
    /// Line number of the finding
    pub line: Option<usize>,
    /// Horizontal position of the finding
    pub width: Option<usize>,
    /// Content around the finding
    pub content: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MetaFinding {
    pub finding: Finding,
    pub meta: Meta,
}

impl MetaFinding {
    #[allow(unused)]
    pub fn format(&self) -> String {
        self.finding.severity.format(format!(
            "{} {}:{} {}",
            self.meta.file.clone(),
            self.meta
                .line
                .map(|line| format!("l{line}"))
                .unwrap_or_default(),
            self.meta.width.unwrap_or_default(),
            self.finding.format()
        ))
    }
}

pub type Findings = HashSet<MetaFinding>;

/// Module name -> Findings
pub type AllFindings = HashMap<String, Findings>;

#[allow(unused)]
pub fn sort_findings_by_len(findings: &AllFindings) -> AllFindings {
    findings
        .clone()
        .iter()
        .map(|(name, mfs)| (name.clone(), mfs.clone()))
        .sorted_by(|(_, mfs1), (_, mfs2)| Ord::cmp(&mfs1.len(), &mfs2.len()))
        .collect()
}

#[derive(Debug, Default)]
pub struct Inside {
    pub function: bool,
    pub unchecked: bool,
    pub for_loop: bool,
    pub while_loop: bool,
    pub constructor: bool,
}
