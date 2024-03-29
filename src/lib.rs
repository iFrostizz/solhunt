pub mod cmd;
pub mod formatter;
pub mod interpreter;
pub mod loader;
pub mod modules;
pub mod solidity;
pub mod utils;
pub mod walker;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
mod test {
    use crate::walker::AllFindings;
    use itertools::Itertools;
    use std::collections::HashSet;

    #[allow(unused)]
    pub fn has_with_module(findings: &AllFindings, name: &str) -> bool {
        match findings.get(name) {
            Some(val) => !val.is_empty(),
            None => false,
        }
    }

    pub fn has_with_code(findings: &AllFindings, name: &str, code: usize) -> bool {
        findings
            .get(name)
            .unwrap_or(&HashSet::new())
            .iter()
            .any(|mf| mf.finding.code == code)
    }

    pub fn has_with_code_file(findings: &AllFindings, file: &str, name: &str, code: usize) -> bool {
        findings
            .get(name)
            .unwrap_or(&HashSet::new())
            .iter()
            .any(|mf| mf.meta.file == file && mf.finding.code == code)
    }

    pub fn has_with_code_at_line(
        findings: &AllFindings,
        file: &str,
        name: &str,
        code: usize,
        line: usize,
    ) -> bool {
        findings
            .get(name)
            .unwrap_or(&HashSet::new())
            .iter()
            .any(|mf| {
                if let Some(l) = mf.meta.line {
                    mf.meta.file == file && mf.finding.code == code && l == line
                } else {
                    false
                }
            })
    }

    #[allow(unused)]
    pub fn findings_with_code_module(findings: &AllFindings, name: &str, code: usize) -> usize {
        findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .count()
    }

    pub fn lines_for_findings_with_code_module(
        findings: &AllFindings,
        name: &str,
        code: usize,
    ) -> Vec<usize> {
        findings
            .get(name)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter(|mf| mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .sorted_by(|a, b| Ord::cmp(a, b))
            .collect()
    }

    #[allow(unused)]
    pub fn lines_for_findings_with_code_file_module(
        findings: &AllFindings,
        file: &str,
        name: &str,
        code: usize,
    ) -> Vec<usize> {
        findings
            .get(name)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter(|mf| mf.meta.file == file && mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .sorted_by(|a, b| Ord::cmp(a, b))
            .collect()
    }
}
