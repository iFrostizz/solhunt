use crate::cmd::parse::parse;

mod cmd;
mod formatter;
mod interpreter;
mod loader;
mod modules;
mod solidity;
mod utils;
mod walker;

#[cfg(test)]
mod tests;

fn main() {
    parse().unwrap();
}

mod test {
    use crate::walker::AllFindings;

    #[cfg(test)]
    pub fn has_with_module(findings: &AllFindings, name: &str) -> bool {
        match findings.get(name) {
            Some(val) => !val.is_empty(),
            None => false,
        }
    }

    #[cfg(test)]
    pub fn has_with_code(findings: &AllFindings, name: &str, code: usize) -> bool {
        findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|mf| mf.finding.code == code)
    }

    #[cfg(test)]
    pub fn has_with_code_file(findings: &AllFindings, file: &str, name: &str, code: usize) -> bool {
        findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|mf| mf.meta.file == file && mf.finding.code == code)
    }

    #[cfg(test)]
    pub fn has_with_code_at_line(
        findings: &AllFindings,
        file: &str,
        name: &str,
        code: usize,
        line: usize,
    ) -> bool {
        findings.get(name).unwrap_or(&Vec::new()).iter().any(|mf| {
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

    #[cfg(test)]
    pub fn lines_for_findings_with_code_module(
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

    #[cfg(test)]
    #[allow(unused)]
    pub fn lines_for_findings_with_code_file_module(
        findings: &AllFindings,
        file: &str,
        name: &str,
        code: usize,
    ) -> Vec<usize> {
        findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|mf| mf.meta.file == file && mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .collect()
    }
}
