use crate::{cmd::parse::parse, utils::formatter::format_findings};
use cmd::parse::get_remappings;
use core::{solidity::Solidity, walker::Walker};
use ethers_solc::{ArtifactId, ConfigurableContractArtifact};
use std::collections::BTreeMap;

mod cmd;
mod modules;
mod utils;

fn main() {
    let (path, loader, verbosity) = parse();

    let solidity = Solidity::default()
        .with_remappings(get_remappings(&path))
        .with_path_root(path);

    // let output = solidity.compile_artifacts();
    let artifacts: BTreeMap<ArtifactId, ConfigurableContractArtifact> = solidity
        .compile_with_foundry()
        .expect("Compilation failed")
        .into_artifacts()
        .collect();

    dbg!(&artifacts);

    let mut walker = Walker::new(artifacts, loader, BTreeMap::new());

    let all_findings = walker.traverse().expect("failed to traverse ast");
    format_findings(all_findings, verbosity);
}

#[cfg(test)]
mod test {
    use crate::modules::loader::get_all_modules;
    use core::{
        loader::Loader,
        solidity::{get_string_lines, ProjectFile},
        walker::{AllFindings, Walker},
    };
    use ethers_solc::{
        artifacts::{
            output_selection::{BytecodeOutputSelection, ContractOutputSelection, OutputSelection},
            Bytecode, Evm, GeneratedSource, Settings,
        },
        error::SolcIoError,
        output::ProjectCompileOutput,
        project_util::TempProject,
        ArtifactId, ConfigurableArtifacts, ConfigurableContractArtifact, Project, ProjectBuilder,
        ProjectPathsConfig, SolcConfig,
    };
    use std::{self, collections::BTreeMap};

    pub fn compile_and_get_findings(files: Vec<ProjectFile>) -> AllFindings {
        // let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();
        let (dir, project) = dapptools_project().unwrap();

        let mut settings = Settings::default();
        settings.output_selection = OutputSelection::default_output_selection();
        settings.push_output_selection("storageLayout");
        settings.push_output_selection("devdoc");
        settings.push_output_selection("userdoc");
        settings = settings.with_ast();

        let project = project
            .solc_config(SolcConfig { settings })
            .set_build_info(true);
        let project = build(dir, project).unwrap();

        let mut source_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();

        files.iter().for_each(|f| {
            let (name, content) = match f {
                ProjectFile::Contract(name, content) => {
                    project.add_source(name, content).unwrap();
                    (name, content)
                }
                ProjectFile::Library(name, content) => {
                    project.add_lib(name, content).unwrap();
                    (name, content)
                }
            };

            source_map.insert(name.clone(), get_string_lines(content.to_string()));
        });

        let compiled = project.compile().unwrap();

        assert!(!compiled.has_compiler_errors());

        println!("---------------------------------");

        // clone is dirty here
        let output = compiled.clone().output();

        let artifacts = compiled
            .into_artifacts()
            .collect::<BTreeMap<ArtifactId, ConfigurableContractArtifact>>()
            .into_iter()
            .find(|(art_id, art)| {
                dbg!(art);

                // dbg!(&art_id.name, &name);
                // let art_id_name = art_id
                //     .source
                //     .clone()
                //     .file_name()
                //     .unwrap()
                //     .to_os_string()
                //     .into_string()
                //     .unwrap();

                // let art_id_name = art_id_name.strip_suffix(".sol").unwrap();

                if let ProjectFile::Contract(name, _) = &files[0] {
                    &art_id.name == name
                } else {
                    false
                }
                // &art_id.name == "Foo"
            })
            .expect("Foo testing contract not found");

        let artifacts = BTreeMap::from([(artifacts.0, artifacts.1)]);

        let modules = get_all_modules();
        let loader = Loader::new(modules);
        let mut walker = Walker::new(artifacts.into(), loader, source_map);

        walker.traverse().expect("failed to traverse ast")
    }

    fn dapptools_project() -> eyre::Result<(tempfile::TempDir, ProjectBuilder)> {
        let tmp_dir = tempdir("tmp_dapp")?;
        let paths = ProjectPathsConfig::dapptools(tmp_dir.path())?;

        Ok((tmp_dir, Project::builder().paths(paths)))
    }

    fn build(
        tmp_dir: tempfile::TempDir,
        project: ProjectBuilder,
    ) -> eyre::Result<TempProject<ConfigurableArtifacts>> {
        let inner = project.build()?;
        Ok(TempProject::create_new(tmp_dir, inner)?)
    }

    fn tempdir(name: &str) -> Result<tempfile::TempDir, SolcIoError> {
        tempfile::Builder::new()
            .prefix(name)
            .tempdir()
            .map_err(|err| SolcIoError::new(err, name))
    }

    // TODO: keep compile_temp but find a solution to read the file, or only pass content
    // pub fn compile_and_get_findings(name: &str, content: &str) -> AllFindings {
    //     let name = name.to_string();
    //     let mut file_name = name.clone();

    //     file_name.push_str(".sol"); // add extension

    //     if fs::create_dir("./test-data/").is_ok() {
    //         println!("I just created the test dir for you")
    //     } // else is probably already here

    //     let root = PathBuf::from("./test-data/");
    //     let path = root.join(file_name.clone());

    //     let mut f = File::create(&path).unwrap();
    //     f.write_all(content.as_bytes()).unwrap();

    //     // dbg!(&root);

    //     let solidity = Solidity::default().with_path_root(root).ephemeral(true);
    //     let output = solidity
    //         .compile()
    //         /*.find_first(name)
    //         .unwrap()
    //         .clone()*/
    //         .into_artifacts()
    //         .collect::<BTreeMap<ArtifactId, ConfigurableContractArtifact>>()
    //         .into_iter()
    //         .find(|(art_id, _)| {
    //             // dbg!(&art_id.name, &name);
    //             // dbg!(art_id);
    //             let art_id_name = art_id
    //                 .source
    //                 .clone()
    //                 .file_name()
    //                 .unwrap()
    //                 .to_os_string()
    //                 .into_string()
    //                 .unwrap();

    //             let art_id_name = art_id_name.strip_suffix(".sol").unwrap();

    //             // dbg!(&art_id_name, &name);

    //             art_id_name == name
    //         })
    //         .unwrap();

    //     let output = BTreeMap::from([(output.0, output.1)]);

    //     let modules = get_all_modules();
    //     let loader = Loader::new(modules);
    //     let mut walker = Walker::new(output.into(), loader);

    //     walker.traverse().expect("failed to traverse ast")
    // }

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

    #[allow(dead_code)]
    pub fn compile_temp(name: impl AsRef<str>, content: impl AsRef<str>) -> ProjectCompileOutput {
        let tmp = TempProject::dapptools().unwrap();
        let f = tmp.add_contract(name, content).unwrap();
        tmp.project().compile_file(f).unwrap()
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
