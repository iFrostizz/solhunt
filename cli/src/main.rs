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
        walker::{AllFindings, Walker},
    };
    use ethers_solc::{output::ProjectCompileOutput, project_util::TempProject};

    pub fn compile_and_get_findings(
        name: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> AllFindings {
        let compiled = compile_temp(name, content);

        assert!(!compiled.has_compiler_errors());
        assert!(compiled.find_first("Foo").is_some());

        let output = compiled.into_artifacts().collect();

        let modules = get_all_modules();
        let loader = Loader::new(modules);

        let mut walker = Walker::new(output, loader);

        walker.traverse().unwrap()
    }

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
}
