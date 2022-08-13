use core::walker::{AllFindings, MetaFinding, Severity};

// TODO: display sort by severity
pub fn format_findings(findings: AllFindings, verbosity: u8) {
    findings.iter().for_each(|(_name, findings)| {
        findings.iter().for_each(|mf| {
            // it means MetaFindings right ? right ???
            if verbosity == 0 { // hm only
                if mf.finding.severity == Severity::High || mf.finding.severity == Severity::Medium {
                    print_finding(mf);
                }
            } else if verbosity == 1 { // hm + gas
                if mf.finding.severity == Severity::High || mf.finding.severity == Severity::Medium || mf.finding.severity == Severity::Low {
                    print_finding(mf);
                }
            } else if verbosity == 2 { // hm + gas + low
                if mf.finding.severity != Severity::Informal {
                    print_finding(mf);
                }
            } else if verbosity >= 3 { // hm + gas + low + info
                print_finding(mf);
            }
        })
    })
}

fn print_finding(mf: &MetaFinding) {
    println!("{}", mf.format());
}
