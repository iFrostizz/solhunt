use crate::cmd::gas::get_gas_diff;
use crate::walker::{AllFindings, Findings, Severity};
use clap::{Parser, ValueEnum};
use cli_table::{print_stdout, Cell, Style, Table};
use itertools::Itertools;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
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
    github: Option<String>,
}

impl Report {
    pub fn new(
        style: ReportStyle,
        root: PathBuf,
        findings: AllFindings,
        verbosity: Vec<Severity>,
        github: Option<String>,
    ) -> Self {
        // only take findings with the chosen verbosity
        let findings = findings
            .into_iter()
            .map(|(name, meta_findings)| {
                (
                    name,
                    meta_findings
                        .into_iter()
                        .filter(|mf| verbosity.contains(&mf.finding.severity))
                        .collect(),
                )
            })
            .collect();

        // sort verbosity from highest
        let verbosity: Vec<_> = verbosity
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
            github,
        }
    }

    pub fn format(&self) {
        match self.style {
            ReportStyle::Md => {
                format_to_md(
                    self.findings.clone(),
                    self.root.clone(),
                    self.verbosity.clone(),
                    self.github.clone(),
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
            ReportStyle::List => {
                todo!()
            } // _ => todo!(),
        }
    }
}

fn format_to_cmd(findings: &AllFindings) -> std::result::Result<(), std::io::Error> {
    let mut tables = Vec::new();

    findings.iter().for_each(|(name, findings)| {
        findings.iter().for_each(|mf| {
            let meta = &mf.meta;
            let position = format!("{}:{}", meta.line.unwrap_or(0), meta.width.unwrap_or(0));

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
    github: Option<String>,
) -> Result<(), std::io::Error> {
    let mut file_path = root;
    file_path.push("report.md");

    let mut buffer = File::create(file_path)?;

    let mut content = String::from("# Solhunt report\n");
    let mut summary = String::from(
        "## Findings summary\nName | Finding | Instances | Gas saved\n--- | --- | --- | ---\n",
    );

    // details of findings including title, summary, description, and view of code
    // for gas, also display the gas savings
    let mut details = String::from("\n## Findings details\n");

    // write the summary in the order of the verbosity
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
            // TODO: unused ?
            // let _title = match v {
            //     Severity::Gas => "## Gas otimizations".to_string(),
            //     Severity::High => "## High severity findings".to_string(),
            //     Severity::Medium => "## Medium severity findings".to_string(),
            //     Severity::Low => "## Low severity findings".to_string(),
            //     Severity::Informal => "## Informal findings".to_string(),
            // };

            let finding_identifier = match v {
                Severity::Gas => "G".to_string(),
                Severity::High => "H".to_string(),
                Severity::Medium => "M".to_string(),
                Severity::Low => "L".to_string(),
                Severity::Informal => "I".to_string(),
            };

            // <(module, code), ((id, f_count, summary), Findings)>
            // <Findings>.len is the count of these findings
            #[allow(clippy::type_complexity)]
            let mut findings_id: HashMap<
                (String, usize),
                ((String, usize, String), Findings),
            > = HashMap::new();

            // tidy up findings and make them unique by grouping them
            these_findings
                .into_iter()
                .for_each(|(module, meta_findings)| {
                    meta_findings.into_iter().for_each(|mf| {
                        findings_id
                            .entry((module.clone(), mf.finding.code))
                            .and_modify(|(_, mfs)| {
                                mfs.insert(mf.clone());
                            })
                            .or_insert_with(|| {
                                findings_count += 1;

                                (
                                    (
                                        finding_identifier.clone(),
                                        findings_count,
                                        mf.finding.summary.clone(),
                                    ),
                                    HashSet::from([mf.clone()]),
                                )
                            });
                    })
                });

            // group findings by their summary
            let findings_id_vec: Vec<_> = findings_id
                .values()
                .sorted_by(|((_, c1, _), _), ((_, c2, _), _)| Ord::cmp(c1, c2))
                .collect();

            findings_id_vec
                .iter()
                .for_each(|((id, f_count, sum), mfs)| {
                    let findings_title =
                        get_table_title(id.clone(), *f_count, sum.clone(), mfs.len(), 0);
                    summary += &findings_title;
                });

            // write details
            findings_id_vec
                .iter()
                .for_each(|((id, f_count, sum), mfs)| {
                    // settle the title
                    let findings_title = get_title(id.to_string(), *f_count, sum.to_string());

                    details.push_str(&format!(
                        "### {}\n\n{}\n\n",
                        findings_title,
                        mfs.iter().next().unwrap().finding.description
                    ));

                    let mut description = String::new();

                    // max amount of code examples given a module giving a lot of them
                    let max_content = 1;

                    // add the description
                    mfs.iter()
                        .enumerate()
                        // TODO: write the comment section and prioritize any comment that 1. doesn't have the same finding code 2. doesn't have the same comment
                        .filter(|(i, _)| i < &max_content)
                        .for_each(|(_, mf)| {
                            let gas_saved = get_gas_diff(
                                mf.finding.name.clone(),
                                mf.finding.code,
                                Default::default(),
                            );

                            let mut formatted_finding = format!(
                                "`{}`\n{}:{}\n\n```solidity\n{}```\n\n",
                                mf.meta.file,
                                mf.meta.line.unwrap_or_default(),
                                mf.meta.width.unwrap_or_default(),
                                mf.meta.content
                            );

                            if let Some(comment) = &mf.finding.comment {
                                formatted_finding
                                    .push_str(&format!("### Comments\n\n{}\n\n", comment));
                            }

                            if let Some(gas_saved) = gas_saved {
                                formatted_finding
                                    .push_str(&format!("### Gas saved\n\n{}\n\n", gas_saved));
                            }

                            description.push_str(&formatted_finding);
                        });

                    if let Some(mut gh) = github.clone() {
                        if !gh.ends_with('/') {
                            gh += "/";
                        }

                        // We should consume the iterator on the first round and keep up with the elements left here after checking that it's not empty
                        if mfs.len() > max_content {
                            description
                                .push_str("<details>\n<summary>Locations</summary>\n<br>\n\n");

                            mfs.iter()
                                .enumerate()
                                .filter(|(i, _)| i >= &max_content)
                                .for_each(|(_, mf)| {
                                    let gh_link = gh.clone()
                                        + &mf.meta.file
                                        + "#L"
                                        + &mf.meta.line.unwrap_or_default().to_string();

                                    description.push_str("- ");
                                    description.push_str(&gh_link);
                                    description.push_str("\n\n");
                                });

                            description.push_str("</details>\n\n")
                        }
                    }

                    details.push_str(&description);
                });
        }
    });

    // push the top summary with the table
    content.push_str(&summary);

    // push all the details to the file
    content.push_str(&details);

    buffer.write_all(content.as_bytes())?;

    Ok(())
}

/// create a table row in a markdown style for the details of the report
fn get_title(id: String, f_code: usize, summary: String) -> String {
    format!("[{}-{}] {}\n", id, f_code, summary)
}

/// create a table row in a markdown style for the summary of the report
fn get_table_title(
    id: String,
    finding_code: usize,
    summary: String,
    instances: usize,
    gas_saved: usize,
) -> String {
    let saved = if gas_saved == 0 {
        String::from("/")
    } else {
        gas_saved.to_string()
    };

    format!(
        "[{}-{}] | {} | {} | {}\n",
        id, finding_code, summary, instances, saved
    )
}
