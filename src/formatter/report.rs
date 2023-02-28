use crate::walker::{AllFindings, Findings, Severity};
use clap::{Parser, ValueEnum};
use cli_table::{print_stdout, Cell, Style, Table};
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;
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
        // sort verbosity from highest
        let verbosity = verbosity
            .into_iter()
            .sorted_by(|v1, v2| {
                let (v1, v2) = (u16::from(*v1), u16::from(*v2));
                Ord::cmp(&v2, &v1)
            })
            .collect();

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

#[allow(unused)]
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

        // only take findings for this specific verbosity
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

        // has at least one finding
        if these_findings.values().any(|mfs| !mfs.is_empty()) {
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

            let mut summary =
                format!("## Findings summary\n{title}\nName | Finding | Instances\n---|---|---\n");

            // <(module, code), ((id, f_count, summary), Findings)>
            // <Findings>.len is the count of these findings
            #[allow(clippy::type_complexity)]
            let mut findings_id: HashMap<
                (String, usize),
                ((String, usize, String), Findings),
            > = HashMap::new();

            these_findings
                .into_iter()
                .for_each(|(module, meta_findings)| {
                    meta_findings.into_iter().for_each(|mf| {
                        findings_id
                            .entry((module.clone(), mf.finding.code))
                            .and_modify(|(_, mfs)| {
                                mfs.push(mf.clone());
                            })
                            .or_insert({
                                let old_count = findings_count;
                                findings_count += 1;

                                (
                                    (finding_identifier.clone(), old_count, summary.clone()),
                                    vec![mf.clone()],
                                )
                            });
                    })
                });

            findings_id
                .iter()
                .for_each(|(_, ((id, f_count, sum), mfs))| {
                    let findings_title = get_title(id.clone(), *f_count, sum.clone(), mfs.len());
                    summary += &findings_title;
                });

            content.push_str(&summary);

            // details of findings including title, summary, description, and view of code
            // for gas, also display the gas savings
            let mut details = String::from("\n## Findings details\n");

            findings_id
                .into_iter()
                .for_each(|(_, ((id, f_count, sum), mfs))| {
                    // settle the title
                    let findings_title = get_title(id, f_count, sum, mfs.len());
                    details.push_str(&("### ".to_owned() + &findings_title));

                    let mut description = String::new();

                    // max amount of code examples given a module giving a lot of them
                    let max_content = 10;

                    // add the description
                    mfs.into_iter()
                        .enumerate()
                        .filter(|(i, _)| i < &max_content)
                        .for_each(|(_, mf)| {
                            // let file = mf.meta.file;
                            // let

                            let formatted_finding = format!(
                                "#### `{}`\n{}:{}\n{}\n```solidity\n{}```\n",
                                mf.meta.file,
                                mf.meta.line.unwrap_or_default(),
                                mf.meta.position.unwrap_or_default(),
                                mf.finding.description,
                                mf.meta.content
                            );
                            // formatted_finding.push_str("\n");

                            description.push_str(&formatted_finding);
                        });

                    details.push_str(&description);
                });

            // push all the details to the file
            content.push_str(&details);
        }
    });

    buffer.write_all(content.as_bytes())?;

    Ok(())
}

fn get_title(id: String, f_count: usize, summary: String, inst_count: usize) -> String {
    format!("[{}-{}] | {} | {}\n", id, f_count, summary, inst_count)
}
