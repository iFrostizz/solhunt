use crate::walker::{AllFindings, MetaFinding, Severity};

// TODO: display sort by severity
pub fn format_findings(findings: AllFindings) {
    findings.iter().for_each(|(_name, findings)| {
        findings.iter().for_each(|mf| {
            print_finding(mf);
        })
    })
}

pub fn filter_findings(findings: AllFindings, verbosity: &[Severity]) -> AllFindings {
    findings
        .into_iter()
        .map(|(name, findings)| {
            (
                name,
                findings
                    .into_iter()
                    .filter(|mf| verbosity.contains(&mf.finding.severity))
                    .collect(),
            )
        })
        .collect()
}

fn print_finding(mf: &MetaFinding) {
    println!("{}", mf.format());
}
