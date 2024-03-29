use crate::{
    loader::get_all_visitors,
    solidity::{build_source_maps, get_finding_content_arrow},
    walker::{AllFindings, Walker},
};
use bytes::Bytes;
use ethers_solc::{
    artifacts::{output_selection::ContractOutputSelection, BytecodeObject, Optimizer, Settings},
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::SolcError,
    output::ProjectCompileOutput,
    project_util::TempProject,
    remappings::{RelativeRemapping, Remapping},
    ArtifactId, ArtifactOutput, ConfigurableArtifacts, ConfigurableContractArtifact, Project,
    ProjectPathsConfig, Solc, SolcConfig,
};
use eyre::{Context, Result};
use glob::glob;
use semver::Version;
use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};
use yansi::Paint;

#[derive(Default)]
pub struct AllFindingsAndSourceMap {
    pub all_findings: AllFindings,
    pub source_map: Vec<usize>,
}

#[allow(unused)]
#[derive(Clone)]
pub enum ProjectFile {
    Contract(String, String),
    Library(String, String),
}

// TODO: use cache and only recompile if files have changed
// TODO: if no svm, display message & start timer after
#[derive(Debug, Clone)]
pub struct Solidity {
    pub root: PathBuf,
    pub allow_paths: Vec<String>,
    pub include_paths: Vec<String>,
    pub extra_output: Vec<ContractOutputSelection>,
    pub extra_output_files: Vec<ContractOutputSelection>,
    pub src: PathBuf,
    pub test: PathBuf,
    pub script: PathBuf,
    pub out: PathBuf,
    pub cache: PathBuf,
    pub libs: Vec<PathBuf>,
    pub remappings: Vec<RelativeRemapping>,
    pub auto_detect_remappings: bool,
    pub libraries: Vec<String>,
    pub use_cache: bool,
    pub build_info_path: Option<PathBuf>,
    pub force: bool,
    pub ephemeral: bool,
    pub solc: Option<Solc>,
    pub optimizer: Optimizer,
    /// stfu ?
    pub silent: bool,
    pub version: Option<Version>,
    pub locations: Option<Vec<PathBuf>>,
}

impl Default for Solidity {
    fn default() -> Self {
        Self {
            root: Default::default(),
            libs: Default::default(),
            allow_paths: Default::default(),
            build_info_path: None,
            include_paths: Default::default(),
            extra_output: Default::default(),
            extra_output_files: Default::default(),
            src: Default::default(),
            test: Default::default(),
            script: Default::default(),
            libraries: Default::default(),
            out: Default::default(),
            cache: Default::default(),
            use_cache: true,
            auto_detect_remappings: false,
            remappings: Default::default(),
            force: false,
            ephemeral: false,
            solc: None,
            optimizer: Default::default(),
            silent: false,
            version: None,
            locations: Default::default(),
        }
    }
}

impl Solidity {
    // // TODO: when foundry uses cache, it does not return the artifacts
    // pub fn compile_with_foundry(&self) -> Result<ProjectCompileOutput> {
    //     // build from single file
    //     let is_contract = self.root.ends_with(".sol");

    //     let project_paths_args = ProjectPathsArgs {
    //         root: if is_contract {
    //             None
    //         } else {
    //             Some(PathBuf::from(&self.root))
    //         },
    //         contracts: if is_contract {
    //             Some(PathBuf::from(&self.root))
    //         } else {
    //             None
    //         },
    //         ..Default::default()
    //     };

    //     let core_build_args = CoreBuildArgs {
    //         // TODO: remove force and use cached artifacts
    //         // If it uses cache, no ProjectCompileOutput will be returned
    //         // so we may need to pull the artifacts
    //         force: true,
    //         silent: false,
    //         project_paths: project_paths_args,
    //         build_info: true,
    //         ..Default::default()
    //     };

    //     let build_args = BuildArgs {
    //         args: core_build_args,
    //         ..Default::default()
    //     };
    //     // dbg!(&build_args.try_load_config_emit_warnings().unwrap());
    //     build_args.run()
    // }

    fn artifacts(&self) -> ConfigurableArtifacts {
        let mut extra_output = self.extra_output.clone();
        // Sourcify verification requires solc metadata output. Since, it doesn't
        // affect the UX & performance of the compiler, output the metadata files
        // by default.
        // For more info see: <https://github.com/foundry-rs/foundry/issues/2795>
        // Metadata is not emitted as separate file because this breaks typechain support: <https://github.com/foundry-rs/foundry/issues/2969>
        if !extra_output.contains(&ContractOutputSelection::Metadata) {
            extra_output.push(ContractOutputSelection::Metadata);
        }

        ConfigurableArtifacts::new(extra_output, self.extra_output_files.clone())
    }

    pub fn get_all_remappings(&self) -> Vec<Remapping> {
        self.remappings.iter().map(|m| m.clone().into()).collect()
    }

    fn project_paths(&self) -> ProjectPathsConfig {
        let mut builder = ProjectPathsConfig::builder()
            .cache(&self.cache)
            .sources(&self.src)
            .tests(&self.test)
            .scripts(&self.script)
            .artifacts(&self.out)
            .libs(self.libs.clone())
            .remappings(self.get_all_remappings());

        if let Some(build_info_path) = &self.build_info_path {
            builder = builder.build_infos(build_info_path);
        }

        builder.build_with_root(&self.root)
    }

    fn solc_settings(&self) -> Settings {
        Settings {
            optimizer: self.optimizer.clone(),
            ..Default::default()
        }
    }

    pub fn project(&self) -> Result<Project, SolcError> {
        let mut project = Project::builder()
            .artifacts(self.artifacts())
            .paths(self.project_paths())
            .allowed_path(&self.root)
            .allowed_paths(&self.libs)
            .allowed_paths(&self.allow_paths)
            // .include_paths(&self.include_paths)
            .solc_config(SolcConfig::builder().settings(self.solc_settings()).build())
            // .ignore_error_codes(self.ignored_error_codes.iter().copied().map(Into::into))
            // .set_compiler_severity_filter(if self.deny_warnings {
            //     Severity::Warning
            // } else {
            //     Severity::Error
            // })
            .set_auto_detect(self.is_auto_detect())
            // .set_offline(self.offline)
            // .set_cached(cached)
            .set_cached(self.use_cache)
            // .set_build_info(cached & self.build_info)
            .set_no_artifacts(false)
            .set_build_info(true);

        if self.ephemeral {
            project = project.ephemeral().no_artifacts();
        }

        let mut project = project.build()?;

        if self.force {
            project.cleanup()?;
        }

        if let Some(solc) = &self.solc {
            project.solc = solc.clone();
        }

        Ok(project)
    }

    #[allow(unused)]
    pub fn with_remappings(mut self, remappings: Vec<RelativeRemapping>) -> Self {
        self.remappings = remappings;
        self
    }

    pub fn with_path_root(self, root: PathBuf) -> Self {
        let root = root.canonicalize().unwrap();
        self.update_root(root)
    }

    pub fn with_version(mut self, version: Version) -> eyre::Result<Self> {
        let solc = Solc::find_or_install_svm_version(version.to_string())?;
        self.solc = Some(solc);
        Ok(self)
    }

    pub fn is_auto_detect(&self) -> bool {
        self.solc.is_none()
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    /// update root and other folders
    fn update_root(mut self, root: PathBuf) -> Self {
        self.root = root;
        self.src = self.root.join("src");
        self.test = self.root.join("test");
        self.script = self.root.join("script");
        self.out = self.root.join("out");
        self.cache = self.root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME);
        self
    }

    #[allow(unused)]
    pub fn with_cache(mut self, cache: PathBuf) -> Self {
        self.cache = cache;
        self
    }

    #[allow(unused)]
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }

    #[allow(unused)]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn auto_remappings(mut self, remappings: bool) -> Self {
        self.auto_detect_remappings = remappings;
        self
    }

    pub fn with_optimizer(mut self, optimizer: Optimizer) -> Self {
        self.optimizer = optimizer;
        self
    }

    pub fn with_locations(mut self, locations: Vec<PathBuf>) -> Self {
        self.locations = Some(locations);
        self
    }

    pub fn use_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    pub fn compile(&mut self) -> Result<ProjectCompileOutput> {
        if self.auto_detect_remappings {
            self.attach_remappings();
        }

        let path = self.root.clone();

        let files = if let Some(locations) = self.locations.clone() {
            locations
        } else if path.is_dir() {
            get_sol_files(&path)
        } else if let Some(ext) = path.extension() {
            if ext == "sol" {
                // walk back to find root and update it
                // TODO: don't use the root variable if it's a single file
                // need to change this!
                let mut root = path.clone();
                root.pop();
                root.pop();

                *self = self.clone().with_path_root(root);

                vec![path]
            } else {
                eyre::bail!("Nothing valid to compile.");
            }
        } else {
            eyre::bail!("Nothing valid to compile.");
        };

        let amount = files.len();
        if !self.silent {
            println!("Compiling {amount} files ...");
        }

        let now = Instant::now();

        let project = &self.project()?;

        let compiled = project.compile_files(files)?;

        if compiled.has_compiler_errors() {
            let output = compiled.output();
            output.errors.iter().for_each(|error| {
                let err_msg = error.formatted_message.clone();
                println!("{}", Paint::red(err_msg.unwrap_or_default()).bold());
            });
            panic!();
        } else if !self.silent {
            println!("Compiled in {}ms\n", now.elapsed().as_millis());
        }

        Ok(compiled)
    }

    pub fn compile_artifacts(
        &mut self,
    ) -> Result<BTreeMap<ArtifactId, ConfigurableContractArtifact>> {
        match self.compile() {
            Ok(compiled) => {
                // let cached = to_cached_artifacts(compiled.into_artifacts().collect())?;
                // Ok(cached)

                Ok(compiled.into_artifacts().collect())
            }
            Err(err) => Err(err),
        }
    }

    fn attach_remappings(&mut self) {
        let mut remappings = Remapping::find_many(&self.root);

        remappings.append(&mut self.remappings_from_file());

        let remappings = remappings
            .into_iter()
            .map(|re| re.into_relative(&self.root))
            .collect();

        self.remappings = remappings;
    }

    fn remappings_from_file(&self) -> Vec<Remapping> {
        let root = PathBuf::from(&self.root).canonicalize().unwrap();
        let mut remap = root.clone();
        remap.push("remappings.txt");

        let remappings_txt = match fs::read_to_string(remap) {
            Ok(content) => content,
            Err(_) => return Vec::new(),
        };

        remappings_txt
            .lines()
            .map(|l| {
                let (name, rpath) = l.split_once('=').unwrap();

                let mut path = root.clone();
                path.push(rpath);
                let path = path
                    .canonicalize()
                    .unwrap_or_else(|e| panic!("{e}: {:#?}", path))
                    .into_os_string()
                    .into_string()
                    .unwrap();

                Remapping {
                    name: name.to_string(),
                    path,
                }
            })
            .collect()
    }
}

/// read compiled artifacts, merge the cached ones
pub fn to_cached_artifacts(
    mut artifacts: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    locations: HashSet<PathBuf>,
) -> Result<BTreeMap<ArtifactId, ConfigurableContractArtifact>> {
    let cart = artifacts.clone();

    for id in cart.keys() {
        let abs_path = id.source.clone();

        if locations.iter().any(|loc| abs_path.starts_with(loc)) {
            let path = &id.path;
            let cached_artifact = Project::<ConfigurableArtifacts>::read_cached_artifact(path)
                .wrap_err_with(|| eyre::eyre!("artifact reading failed for path: {:?}", path))?;

            artifacts.insert(id.clone(), cached_artifact);
        } else {
            // remove any out-of-scope artifacts
            artifacts.remove(id);
        }
    }

    Ok(artifacts)
}

// get path of all .sol files
pub fn get_sol_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    visit_dirs(path, &mut files).expect("failed to get contracts");

    files
}

// could do caching, but explicitely excluding directory is probably good enough ?
pub fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
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
                files.push(path.clone());
            }
        }
    }

    Ok(())
}

fn is_sol_file(path: &Path) -> bool {
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

pub fn get_glob_locations(path: &str) -> eyre::Result<HashSet<PathBuf>> {
    let glob = glob(path)?.collect::<Result<HashSet<_>, _>>()?;
    if glob.is_empty() {
        eyre::bail!("No path matched by glob")
    }

    Ok(glob)
}

pub fn compile_path_to_artifacts(
    path: &str,
    optimizer: Option<Optimizer>,
) -> eyre::Result<BTreeMap<ArtifactId, ConfigurableContractArtifact>> {
    let root = PathBuf::from(path).canonicalize().unwrap();

    let mut solidity = Solidity::default()
        .with_path_root(root.clone())
        .auto_remappings(true)
        .silent();

    if let Some(optimizer) = optimizer {
        solidity = solidity.with_optimizer(optimizer);
    }

    let compiled = solidity.compile().unwrap();

    let root_str = root.into_os_string().into_string().unwrap();
    let glob: HashSet<_> = get_glob_locations(&(root_str + "/**/*.sol"))?;

    to_cached_artifacts(
        compiled
            .into_artifacts()
            .collect::<BTreeMap<ArtifactId, ConfigurableContractArtifact>>(),
        glob,
    )
}

// TODO: optional glob path, else get all sol files
#[cfg(test)]
pub fn compile_path_and_get_findings(
    path: &str,
    optimizer: Option<Optimizer>,
) -> eyre::Result<AllFindings> {
    let artifacts = compile_path_to_artifacts(path, optimizer)?;

    let source_map = super::build_artifacts_source_maps(&artifacts);
    if source_map.is_empty() {
        eyre::bail!("No source map")
    }

    if let Some(debug) = env::var_os("DEBUG") {
        if debug == "true" || debug == "True" || debug == "TRUE" {
            artifacts.iter().for_each(|(_, art)| {
                if let Some(ast) = &art.ast {
                    println!("{:#?}", ast.clone().to_typed());
                }
            });
        }
    };

    let visitors = get_all_visitors();

    let mut walker = Walker::new(artifacts, source_map, visitors);

    walker.traverse()
}

pub fn compile_contract_and_get_findings(contract: String) -> AllFindings {
    compile_and_get_findings(vec![ProjectFile::Contract(String::from("A"), contract)])
}

// TODO: make an easier version that give it a random contract name if there is only one
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

    let visitors = get_all_visitors();

    let mut walker = Walker::new(artifacts, source_map, visitors);

    walker.traverse().expect("failed to traverse ast")
}

#[allow(unused)]
pub fn compile_locations(
    root: PathBuf,
    locations: Vec<PathBuf>,
    version: Version,
) -> eyre::Result<BTreeMap<ArtifactId, ConfigurableContractArtifact>> {
    let mut solidity = Solidity::default()
        .with_path_root(root)
        .with_locations(locations)
        .with_version(version)?;

    solidity.compile_artifacts()
}

#[allow(unused)]
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
        bytecode.to_vec().into()
    } else {
        panic!("No bytecode found");
    }
}

#[cfg(test)]
pub fn compile_single_contract_to_artifacts(
    contract: String,
) -> (
    TempProject<ConfigurableArtifacts>,
    BTreeMap<ArtifactId, ConfigurableContractArtifact>,
) {
    let files = vec![ProjectFile::Contract(
        String::from("SingleContract"),
        contract,
    )];
    let (project, compiled) = make_temp_project(files);

    let artifacts = compiled.into_artifacts().collect();

    (project, artifacts)
}

#[allow(unused)]
pub fn compile_single_contract_to_artifacts_path(
    path: PathBuf,
    version: Version,
) -> Result<BTreeMap<ArtifactId, ConfigurableContractArtifact>> {
    let mut solidity = Solidity::default()
        .with_path_root(path)
        .with_version(version)?
        .silent();

    solidity.compile_artifacts()
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
            println!("{:#?}", err.message);

            let source = err
                .source_location
                .clone()
                .expect("Failed to build debug source location");

            let file_path = err
                .source_location
                .clone()
                .expect("Could not find source location for content debug")
                .file;

            let mut contract_iter = files
                .clone()
                .into_iter()
                .map(|p_file| match p_file {
                    ProjectFile::Contract(f, n) => (f, n),
                    ProjectFile::Library(f, n) => (f, n),
                })
                .filter(|(f, _)| {
                    let mut path = String::from("src/");
                    path.push_str(f);
                    path.push_str(".sol");

                    path == file_path
                });

            assert!(contract_iter.clone().count() > 0);

            let contract = contract_iter.next().unwrap().1;

            let content = if source.start == -1 || source.end == -1 {
                String::from("")
            } else {
                get_finding_content_arrow(
                    contract,
                    source.start.try_into().unwrap(),
                    (source.end - source.start).try_into().unwrap(),
                )
            };

            println!("{content}");
        });
        panic!("Please fix compiler errors first");
    }

    (project, compiled)
}
