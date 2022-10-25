use ethers_solc::{
    output::ProjectCompileOutput,
    remappings::Remapping,
    // ProjectPathsConfig, Solc,
    ArtifactId,
    ConfigurableContractArtifact,
    Project,
};
use std::{
    collections::btree_map::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

// TODO: use cache and only recompile if files have changed
// TODO: if no svm, display message & start timer after

pub fn compile(
    auto_detect: bool,
    path: PathBuf,
    remappings: Vec<Remapping>,
) -> ProjectCompileOutput {
    let mut project = Project::builder().build().unwrap();
    project.paths.remappings = remappings;

    let files = if path.is_dir() {
        get_sol_files(path)
    } else if let Some(ext) = path.extension() {
        if ext == "sol" {
            vec![path]
        } else {
            panic!("Nothing valid to compile.");
        }
    } else {
        panic!("Nothing valid to compile.");
    };

    let amount = files.len();
    println!("Compiling {} files ...", amount);

    let now = Instant::now();

    let compiled = if auto_detect {
        // project.compile().unwrap()
        project.compile_files(files).unwrap()
    } else {
        /*let sources = project.paths.read_sources().unwrap();
        project
            .compile_with_version(
                &Solc::find_svm_installed_version("0.8.0").unwrap().unwrap(),
                sources,
            )
            .unwrap()*/
        unimplemented!();
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
    path: &Path,
    remappings: Vec<Remapping>,
) -> BTreeMap<ArtifactId, ConfigurableContractArtifact> {
    let compiled = compile(auto_detect, path.to_path_buf(), remappings);

    compiled.into_artifacts().collect()
}

// get path of all .sol files
pub fn get_sol_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    visit_dirs(path.as_path(), &mut files).expect("failed to get contracts");

    files
}

// could do caching, but explicitely excluding directory is probably good enough ?
pub fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> eyre::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if !(dir.ends_with("lib") // don't even try to go in libs, cache, etc...
                    || dir.ends_with("node_modules")
                    || dir.ends_with("out")
                    || dir.ends_with("cache")
                    || dir.ends_with("target")
                    || dir.ends_with("artifacts"))
                {
                    visit_dirs(&path, files)?;
                }
            } else if is_sol_file(&path) {
                files.push(path);
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
                    if let Some(str) = path.to_str() {
                        if !(str.ends_with(".t.sol") || str.ends_with(".s.sol")) {
                            // not a test or a script
                            return true;
                        }
                    }
                }
            }
            _ => return false,
        }
    }

    false
}
