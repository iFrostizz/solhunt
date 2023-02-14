use crate::cmd::parse::parse;

mod cmd;
mod formatter;
mod interpreter;
mod loader;
mod modules;
mod solidity;
mod utils;
mod walker;

fn main() {
    parse();
}

mod test {
    use crate::walker::AllFindings;

    pub fn has_with_module(findings: &AllFindings, name: &str) -> bool {
        match findings.get(name) {
            Some(val) => !val.is_empty(),
            None => false,
        }
    }

    pub fn has_with_code(findings: &AllFindings, name: &str, code: usize) -> bool {
        findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|mf| mf.finding.code == code)
    }

    #[allow(dead_code)]
    pub fn has_with_code_at_line(
        findings: &AllFindings,
        name: &str,
        code: usize,
        line: usize,
    ) -> bool {
        findings.get(name).unwrap_or(&Vec::new()).iter().any(|mf| {
            if let Some(l) = mf.meta.line {
                mf.finding.code == code && l == line
            } else {
                false
            }
        })
    }

    #[allow(dead_code)]
    pub fn findings_with_code(findings: &AllFindings, name: &str, code: usize) -> usize {
        findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .count()
    }

    pub fn lines_for_findings_with_code(
        findings: &AllFindings,
        name: &str,
        code: usize,
    ) -> Vec<usize> {
        findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|mf| mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .collect()
    }
}
