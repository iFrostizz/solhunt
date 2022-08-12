// Store findings and do whatever with them

use std::collections::HashMap;

#[derive(Debug)]
pub enum Severity {
    Informal,
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn format(&self, text: String) -> String {
        match *self {
            Severity::Informal => format!("\x1b[0m{}\x1b[0m", text),
            Severity::Low => format!("\x1b[0m{}\x1b[0m", text),
            Severity::Medium => format!("\x1b[0m{}\x1b[0m", text),
            Severity::High => format!("\x1b[0m{}\x1b[0m", text),
        }
    }
}

#[derive(Debug)]
pub struct Finding {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    // TODO: add finding code to identify
}

impl Finding {
    pub fn format(&self) -> String {
        self.severity.format(format!(
            "{}: {}",
            self.name.clone(),
            self.description.clone()
        ))
    }
}

#[derive(Debug)]
pub struct Meta {
    pub file: String,
    pub src: Option<usize>,
}

#[derive(Debug)]
pub struct MetaFinding {
    pub finding: Finding,
    pub meta: Meta,
}

impl MetaFinding {
    pub fn format(&self) -> String {
        format!(
            "{} l.{} {}",
            self.meta.file.clone(),
            self.meta.src.unwrap_or(0),
            self.finding.format()
        )
    }
}

pub type Findings = Vec<MetaFinding>;

/// Module name -> Findings
pub type AllFindings = HashMap<String, Findings>;
