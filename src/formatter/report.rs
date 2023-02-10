use crate::walker::{AllFindings, Severity};
use clap::{Parser, ValueEnum};
use cli_table::{print_stdout, Cell, Style, Table};
use revm::primitives::HashMap;
use serde::Serialize;
use std::{fmt::Debug, fs::File, io::Write, path::PathBuf, str::FromStr};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Parser, ValueEnum)]
pub enum ReportStyle {
    List,
    Cmd,
    Md,
    Html,
}

impl Default for ReportStyle {
    fn default() -> Self {
        Self::List
    }
}

impl FromStr for ReportStyle {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "list" => Ok(ReportStyle::List),
            "cmd" => Ok(ReportStyle::Cmd),
            "md" => Ok(ReportStyle::Md),
            "html" => Ok(ReportStyle::Html),
            _ => Err("Wrong report style provided".to_string()),
        }
    }

    type Err = String;
}

#[derive(Debug)]
pub struct Report {
    root: PathBuf,
    style: ReportStyle,
    findings: AllFindings,
    verbosity: Vec<Severity>,
}

impl Report {
    pub fn new(
        style: ReportStyle,
        root: PathBuf,
        findings: AllFindings,
        verbosity: Vec<Severity>,
    ) -> Self {
        Self {
            style,
            root,
            findings,
            verbosity,
        }
    }

    pub fn format(&self) {
        match self.style {
            ReportStyle::Md => {
                format_to_md(
                    self.findings.clone(),
                    self.root.clone(),
                    self.verbosity.clone(),
                )
                .unwrap();

                println!("Finished writing the markdown report!");
            }
            ReportStyle::Html => {
                todo!()
            }
            ReportStyle::Cmd => {
                format_to_cmd(&self.findings).unwrap();
            }
            _ => todo!(),
        }
    }
}

fn format_to_cmd(findings: &AllFindings) -> std::result::Result<(), std::io::Error> {
    let mut tables = Vec::new();

    findings.iter().for_each(|(name, findings)| {
        findings.iter().for_each(|mf| {
            let meta = &mf.meta;
            let position = format!("{}:{}", meta.line.unwrap_or(0), meta.position.unwrap_or(0));
            tables.push(vec![
                name.cell(),
                mf.finding.severity.cell(),
                mf.finding.summary.clone().cell(),
                // mf.finding.src.cell(),
                position.cell(),
            ]);
        })
    });

    let tables = tables
        .table()
        .title(vec![
            "module".cell().bold(true),
            "severity".cell().bold(true),
            "finding".cell().bold(true),
            "position".cell().bold(true),
        ])
        .bold(true);

    print_stdout(tables)
}

pub struct Instances {
    /// Identification of the file, has form "src/some/path/Contract.sol"
    pub file: String,
    /// Lines of the instance
    pub line: Option<usize>,
    /// Number of instance of this finding
    pub count: usize,
}

/// Write a markdown format of the findings
fn format_to_md(
    findings: AllFindings,
    root: PathBuf,
    verbosity: Vec<Severity>,
) -> Result<(), std::io::Error> {
    let mut file_path = root;
    file_path.push("report.md");

    let mut buffer = File::create(file_path)?;

    let mut content = String::from("# Solhunt report\n");

    verbosity.iter().for_each(|v| {
        let mut findings_count: usize = 0;

        let these_findings: AllFindings = findings
            .clone()
            .into_iter()
            .map(|(name, meta_findings)| {
                (
                    name,
                    meta_findings
                        .into_iter()
                        .filter(|mf| mf.finding.severity == *v)
                        .collect(),
                )
            })
            .collect();

        let has_some = these_findings.values().any(|mfs| !mfs.is_empty());

        if has_some {
            let title = match v {
                Severity::Gas => "## Gas otimizations".to_string(),
                Severity::High => "## High severity findings".to_string(),
                Severity::Medium => "## Medium severity findings".to_string(),
                Severity::Low => "## Low severity findings".to_string(),
                Severity::Informal => "## Informal findings".to_string(),
            };

            let finding_identifier = match v {
                Severity::Gas => "G".to_string(),
                Severity::High => "H".to_string(),
                Severity::Medium => "M".to_string(),
                Severity::Low => "L".to_string(),
                Severity::Informal => "I".to_string(),
            };

            let mut summary = format!("{title}\nName | Finding | Instances\n---|---|---\n");

            let mut findings_dedup: HashMap<String, Instances> = HashMap::new();

            // group each finding per summary and count them
            these_findings.into_iter().for_each(|(_, meta_findings)| {
                meta_findings.into_iter().for_each(|mf| {
                    findings_dedup
                        .entry(mf.finding.summary)
                        .and_modify(|instance| instance.count += 1)
                        .or_insert(Instances {
                            file: mf.meta.file,
                            line: mf.meta.line,
                            count: 0,
                        });
                })
            });

            findings_dedup.into_iter().for_each(|(sum, inst)| {
                summary += &format!(
                    "[{}-{}] | {} | {}\n",
                    finding_identifier, findings_count, sum, inst.count
                );
                findings_count += 1;
            });

            content.push_str(&summary);
        }
    });

    buffer.write_all(content.as_bytes())?;

    Ok(())
}
