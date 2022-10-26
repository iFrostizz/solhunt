use cmd::parse::get_remappings;

use crate::{cmd::parse::parse, utils::formatter::format_findings};
use core::{solidity, walker::Walker};

mod cmd;
mod modules;
mod utils;

fn main() {
    let (path, loader, verbosity) = parse();

    let output = solidity::compile_artifacts(true, &path, get_remappings(&path));

    let mut walker = Walker::new(output, loader);

    let all_findings = walker.traverse().expect("failed to traverse ast");
    format_findings(all_findings, verbosity);
}

#[cfg(test)]
mod test {
    use crate::modules::loader::get_all_modules;
    use core::{
        loader::Loader,
        solidity,
        walker::{AllFindings, Findings, MetaFinding, Walker},
    };
    use ethers_solc::{output::ProjectCompileOutput, project_util::TempProject};
    use std::{fs::File, io::Write, path::Path};

    // TODO: keep compile_temp but find a solution to read the file, or only pass content
    pub fn compile_and_get_findings(name: &str, content: &str) -> AllFindings {
        let mut name = name.to_string();
        name.push_str(".sol"); // add extension
        let path = Path::new("./test-data/").join(name);

        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();

        let output = solidity::compile_artifacts(true, &path, Default::default());

        let modules = get_all_modules();
        let loader = Loader::new(modules);
        let mut walker = Walker::new(output, loader);

        walker.traverse().expect("failed to traverse ast")
    }

    /*pub fn compile_and_get_findings(
        name: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> AllFindings {
        let compiled = compile_temp(name, content);

        assert!(!compiled.has_compiler_errors());
        assert!(compiled.find_first("Foo").is_some());

        let output = compiled.into_artifacts().collect();

        let mut f = File::create(name.try_into()
        .unwrap()).unwrap();
        let con: String = content.try_into().unwrap();
        f.write_all(con.as_bytes()).unwrap();
        f.sync_all().unwrap();

        let modules = get_all_modules();
        let loader = Loader::new(modules);

        let mut walker = Walker::new(output, loader);

        walker.traverse().unwrap()
    }*/

    pub fn compile_temp(name: impl AsRef<str>, content: impl AsRef<str>) -> ProjectCompileOutput {
        let tmp = TempProject::dapptools().unwrap();
        let f = tmp.add_contract(name, content).unwrap();
        tmp.project().compile_file(f).unwrap()
    }

    // TODO: be more specific with file line and multiple findings
    pub fn has_with_code(all_findings: &AllFindings, name: &str, code: u32) -> bool {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .any(|mf| mf.finding.code == code)
    }

    pub fn has_with_code_at_line(
        all_findings: &AllFindings,
        name: &str,
        code: u32,
        line: u32,
    ) -> bool {
        all_findings.get(name).unwrap().iter().any(|mf| {
            if let Some(l) = mf.meta.line {
                mf.finding.code == code && l == line
            } else {
                false
            }
        })
    }

    /*pub fn get_findings_with_code_at_line(
        all_findings: &AllFindings,
        name: &str,
        code: u32,
    ) -> Vec<MetaFinding> {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .collect::<Vec<MetaFinding>>()
    }*/

    pub fn findings_with_code(all_findings: &AllFindings, name: &str, code: u32) -> u32 {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .count() as u32
    }

    pub fn lines_for_findings_with_code(
        all_findings: &AllFindings,
        name: &str,
        code: u32,
    ) -> Vec<u32> {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .collect()
    }
}
