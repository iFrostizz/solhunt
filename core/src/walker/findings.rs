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

pub type Findings = Vec<Finding>;

pub type AllFindings = HashMap<String, Findings>;
