use ethers_solc::{
    output::ProjectCompileOutput, ArtifactId, ConfigurableContractArtifact, Project,
    ProjectPathsConfig, Solc,
};
use std::{collections::btree_map::BTreeMap, path::{PathBuf, Path}, time::Instant, fs};

// TODO: implement compile files https://docs.rs/ethers-solc/latest/ethers_solc/struct.Project.html#method.compile_files

pub fn compile(auto_detect: bool, path: PathBuf) -> ProjectCompileOutput {
    /*let sources = if path.as_path().join("contracts").as_path().exists() {
        if path.as_path().join("src").as_path().exists() {
            vec![path.as_path().join("contracts"), path.as_path().join("src")]
        } else {
            vec![path.as_path().join("contracts")]
        }
    } else {
        if path.as_path().join("src").as_path().exists() {
            vec![path.as_path().join("src")]
        } else {
            vec![path]
        }
    };*/ // TODO: find solution

    let paths = ProjectPathsConfig::builder()
        .sources(&path)
        .libs(ProjectPathsConfig::find_libs(&path))
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
            )
            .unwrap()
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

pub fn compile_artifacts(
    auto_detect: bool,
    path: PathBuf,
) -> BTreeMap<ArtifactId, ConfigurableContractArtifact> {
    let compiled = compile(auto_detect, path);

    compiled.into_artifacts().collect()
}

// get path of all .sol files
/*pub fn get_sol_files<'a>(path: PathBuf) -> Vec<&'a Path> {
    let mut files = Vec::new();

    visit_dirs(path.as_path(), &mut files).expect("failed to get contracts");

    files
}*/

pub fn visit_dirs(dir: &Path, files: &mut Vec<&Path>) -> eyre::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else {
                if is_sol_file(&path) {
                    files.push(&path);
                }
            }
        }
    }

    Ok(())
}

pub fn is_sol_file(path: &Path) -> bool {
    if path.is_file() {
        match path.extension() {
            Some(extension) => {
                if extension == "sol" {
                    if !(path.ends_with(".t.sol") || path.ends_with(".s.sol")) { // not a test or a script
                        return true;
                    }
                }
            }
            _ => return false,
        }
    }

    return false;
}
