use crate::{
    cmd::parse::parse,
    loader::ModuleFindings,
    solidity::{get_path_lines, Solidity},
    utils::formatter::format_findings,
    walker::{Finding, Walker},
};
use cmd::parse::get_remappings;
use ethers_solc::AggregatedCompilerOutput;
use modules::uint256::Uint256Module;
use std::collections::BTreeMap;

mod cmd;
mod loader;
mod modules;
mod solidity;
mod utils;
mod walker;

fn main() {
    // TODO: configurable
    let included_folders: Vec<String> = vec![String::from("src")];

    let (path, loader, verbosity) = parse();

    let solidity = Solidity::default()
        .with_remappings(get_remappings(&path))
        .with_path_root(path.clone());

    let compiled = solidity.compile_with_foundry().expect("Compilation failed");
    let output = compiled.clone().output();

    let source_map = build_source_maps(output);

    let artifacts = compiled
        .into_artifacts()
        .filter(|(id, _art)| {
            let root_path = &path;
            if root_path.is_dir() {
                // only filter if not "file-only"
                let abs_path = &id.source;
                let other_path = abs_path
                    .strip_prefix(root_path)
                    .expect("Failed to strip root path");
                let first_folder = other_path
                    .iter()
                    .next()
                    .expect("Failed to get first folder");
                // only take included folders
                included_folders.contains(&first_folder.to_string_lossy().to_string())
            } else {
                false
            }
        })
        .collect();

    let visitor = Uint256Module::default();
    let mut walker = Walker::new(artifacts, loader, source_map, visitor);

    let all_findings = walker.traverse().expect("failed to traverse ast");
    format_findings(all_findings, verbosity);
}

fn build_source_maps(output: AggregatedCompilerOutput) -> BTreeMap<String, Vec<usize>> {
    output
        .contracts
        .iter()
        .map(|(id, _)| {
            let abs_path = id.to_string();
            (
                abs_path.clone(),
                get_path_lines(abs_path.clone())
                    .unwrap_or_else(|_| panic!("Source map failed for {}", &abs_path)),
            )
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        loader::Loader,
        modules::loader::get_all_modules,
        solidity::ProjectFile,
        walker::{AllFindings, Walker},
    };
    use ethers_solc::{
        project_util::TempProject, ArtifactId, ConfigurableArtifacts, ConfigurableContractArtifact,
    };
    use std::{self, collections::BTreeMap};

    /// Tests utils to compile a temp project similar to reality
    pub fn compile_and_get_findings(files: Vec<ProjectFile>) -> AllFindings {
        let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

        files.iter().for_each(|f| match f {
            ProjectFile::Contract(name, content) => {
                project.add_source(name, content).unwrap();
            }
            ProjectFile::Library(name, content) => {
                project.add_lib(name, content).unwrap();
            }
        });

        let compiled = project.compile().unwrap();

        assert!(!compiled.has_compiler_errors());

        // clone is dirty here
        let output = compiled.clone().output();

        let source_map = build_source_maps(output);

        let artifacts = compiled
            .into_artifacts()
            .collect::<BTreeMap<ArtifactId, ConfigurableContractArtifact>>();

        let modules = get_all_modules();
        let loader = Loader::new(modules);
        let visitor = Uint256Module::default();
        let mut walker = Walker::new(artifacts, loader, source_map, visitor);

        walker.traverse().expect("failed to traverse ast")
    }

    pub fn has_with_module(all_findings: &AllFindings, name: &str) -> bool {
        !all_findings.get(name).unwrap().is_empty()
    }

    // TODO: be more specific with file line and multiple findings
    pub fn has_with_code(all_findings: &AllFindings, name: &str, code: u32) -> bool {
        all_findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|mf| mf.finding.code == code)
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|mf| mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .collect()
    }
}
