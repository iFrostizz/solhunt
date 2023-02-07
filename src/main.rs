use crate::{
    cmd::parse::parse,
    formatter::{Report, ReportStyle},
    // loader::get_all_visitors,
    solidity::{get_path_lines, Solidity},
    utils::{filter_findings, formatter::format_findings},
    walker::Walker,
};
use cmd::parse::get_remappings;
use ethers_solc::AggregatedCompilerOutput;
use loader::get_all_visitors;
use std::collections::BTreeMap;

mod cmd;
mod formatter;
mod interpreter;
mod loader;
mod modules;
mod solidity;
mod utils;
mod walker;

fn main() {
    // TODO: configurable with glob
    let included_folders: Vec<String> = vec![String::from("src")];

    let (path, verbosity, report_style) = parse();

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

    let visitors = get_all_visitors();

    let mut walker = Walker::new(artifacts, source_map, visitors);

    println!("Starting the analysis...");

    let findings = walker.traverse().expect("failed to traverse ast");
    let num_findings = findings.len();
    println!("Caught {num_findings} findings");

    // let findings = filter_findings(findings, &verbosity);
    // format_findings(findings);

    let report = Report::new(report_style, path, findings, verbosity);
    report.format();
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
                    .unwrap_or_else(|e| panic!("Source map failed for {}. {}", &abs_path, e)),
            )
        })
        .collect()
}

mod test {
    use super::*;
    use crate::{
        solidity::ProjectFile,
        walker::{AllFindings, Walker},
    };
    use ethers_core::abi::ethabi::Bytes;
    use ethers_solc::{
        artifacts::BytecodeObject, project_util::TempProject, ArtifactId, ConfigurableArtifacts,
        ConfigurableContractArtifact, ProjectCompileOutput,
    };
    use std::{self, collections::BTreeMap, env};

    /// Tests utils to compile a temp project similar to reality
    pub fn compile_and_get_findings(files: Vec<ProjectFile>) -> AllFindings {
        let (_project, compiled) = make_temp_project(files);
        let output = compiled.clone().output();

        let source_map = build_source_maps(output);

        let artifacts = compiled
            .into_artifacts()
            .collect::<BTreeMap<ArtifactId, ConfigurableContractArtifact>>();

        if let Some(debug) = env::var_os("DEBUG") {
            if debug == "true" || debug == "True" || debug == "TRUE" {
                // println!("{:#?}", project.root);
                artifacts.iter().for_each(|(_, art)| {
                    // println!("{:#?}", art.ast);
                    if let Some(ast) = &art.ast {
                        println!("{:#?}", ast.clone().to_typed());
                    }
                });
            }
        };

        // let visitors: Vec<
        //     Box<(dyn ethers_solc::artifacts::visitor::Visitor<Vec<Finding>> + 'static)>,
        // > = get_all_visitors!("./modules");

        let visitors = get_all_visitors();

        let mut walker = Walker::new(artifacts, source_map, visitors);

        walker.traverse().expect("failed to traverse ast")
    }

    pub fn compile_single_contract(contract: String) -> Bytes {
        let files = vec![ProjectFile::Contract(
            String::from("SingleContract"),
            contract,
        )];
        let (_project, compiled) = make_temp_project(files);
        let output = compiled.output();
        let ver_contracts = output.contracts;

        assert_eq!(ver_contracts.len(), 1);

        let contracts = ver_contracts.iter().next().unwrap().1;

        assert_eq!(contracts.len(), 1);

        let contract = &contracts.iter().next().unwrap().1[0].contract;
        let bytecode = contract.evm.clone().unwrap().bytecode.unwrap();

        if let BytecodeObject::Bytecode(bytecode) = bytecode.object {
            bytecode.to_vec()
        } else {
            panic!("No bytecode found");
        }
    }

    /// Creates a temp project and compiles the files in it
    /// Note: returns the ownership of Project not to be dropped and deleted
    fn make_temp_project(
        files: Vec<ProjectFile>,
    ) -> (TempProject<ConfigurableArtifacts>, ProjectCompileOutput) {
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

        if compiled.has_compiler_errors() {
            compiled.output().errors.iter().for_each(|err| {
                // TODO: write line and position with err.src
                println!("{:#?} {:#?}", err.message, err.source_location);
            });
            panic!("Please fix compiler errors first");
        }

        (project, compiled)
    }

    #[allow(unused)]
    pub fn has_with_module(all_findings: &AllFindings, name: &str) -> bool {
        match all_findings.get(name) {
            Some(val) => !val.is_empty(),
            None => false,
        }
    }

    #[allow(unused)]
    pub fn has_with_code(all_findings: &AllFindings, name: &str, code: usize) -> bool {
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
        code: usize,
        line: usize,
    ) -> bool {
        all_findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|mf| {
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
        code: usize,
    ) -> Vec<MetaFinding> {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .collect::<Vec<MetaFinding>>()
    }*/

    #[allow(dead_code)]
    pub fn findings_with_code(all_findings: &AllFindings, name: &str, code: usize) -> usize {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .filter(|mf| mf.finding.code == code)
            .count()
    }

    pub fn lines_for_findings_with_code(
        all_findings: &AllFindings,
        name: &str,
        code: usize,
    ) -> Vec<usize> {
        all_findings
            .get(name)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|mf| mf.finding.code == code)
            .filter_map(|mf| mf.meta.line)
            .collect()
    }
}
