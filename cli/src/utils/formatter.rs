use core::walker::AllFindings;

// TODO: display by severity
pub fn format_findings(findings: AllFindings) {
    findings.iter().for_each(|(name, findings)| {
        findings.iter().for_each(|mf| {
            // it means MetaFindings right ? right ???
            println!("{}", mf.format());
        })
    })
}
