use crate::walker::{AllFindings, MetaFinding, Severity};

// TODO: display sort by severity
pub fn format_findings(findings: AllFindings, verbosity: Vec<Severity>) {
    findings.iter().for_each(|(_name, findings)| {
        findings.iter().for_each(|mf| {
            if verbosity.contains(&mf.finding.severity) {
                print_finding(mf);
            }
        })
    })
}

fn print_finding(mf: &MetaFinding) {
    println!("{}", mf.format());
}
