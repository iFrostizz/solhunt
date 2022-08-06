use ethers_solc::{Project, ProjectPathsConfig, Solc, ConfigurableContractArtifact, ArtifactId, output::ProjectCompileOutput};
use std::{path::PathBuf, collections::btree_map::BTreeMap, time::Instant};

// TODO: implement compile files https://docs.rs/ethers-solc/latest/ethers_solc/struct.Project.html#method.compile_files

pub fn compile(auto_detect: bool, path: PathBuf) -> ProjectCompileOutput {
    let paths = ProjectPathsConfig::builder()
        .sources(path)
        // .lib(root.join("lib"))
        .build()
        .unwrap();

    let project = Project::builder().paths(paths).build().unwrap();

    let files = project.sources().unwrap().keys().len();
    println!("Compiling {} files ...", files); // TODO: use the pre-built method

    let now = Instant::now();

    let compiled = if auto_detect {
        project.compile().unwrap()
    } else {
        let sources = project.paths.read_sources().unwrap();
        project
            .compile_with_version(
                &Solc::find_svm_installed_version("0.8.0").unwrap().unwrap(),
                sources,
            ).unwrap()
    };

    // project.rerun_if_sources_changed();

    println!("Compiled in {}ms", now.elapsed().as_millis());

    if compiled.has_compiler_errors() {
        let output = compiled.output();
        output.errors.iter().for_each(|error| {
            println!("{:#?}", error.formatted_message);
        });
        panic!();
    }

    compiled
}

pub fn compile_artifacts(auto_detect: bool, path: PathBuf) -> BTreeMap<ArtifactId, ConfigurableContractArtifact> {
    let compiled = compile(auto_detect, path);

    compiled.into_artifacts().collect()
}
