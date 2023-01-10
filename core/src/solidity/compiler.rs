use crate::walker::AllFindings;
use ethers_solc::{
    artifacts::output_selection::ContractOutputSelection,
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::SolcError,
    output::ProjectCompileOutput,
    remappings::{RelativeRemapping, Remapping},
    // ProjectPathsConfig, Solc,
    ArtifactId,
    ConfigurableArtifacts,
    ConfigurableContractArtifact,
    Project,
    ProjectPathsConfig,
};
use foundry_cli::{
    cmd::{
        forge::{
            build::{BuildArgs, CoreBuildArgs, ProjectPathsArgs},
            install,
        },
        Cmd,
    },
    opts::forge::Subcommands,
};
use foundry_common::compile::{self, ProjectCompiler};
use foundry_config::Config;
use std::{
    collections::btree_map::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct AllFindingsAndSourceMap {
    pub all_findings: AllFindings,
    pub source_map: Vec<usize>,
}

pub enum ProjectFile {
    Contract(String, String),
    Library(String, String),
}

// TODO: use cache and only recompile if files have changed
// TODO: if no svm, display message & start timer after

pub struct Solidity {
    pub root: String,
    pub allow_paths: Vec<String>,
    pub include_paths: Vec<String>,
    pub extra_output: Vec<ContractOutputSelection>,
    pub extra_output_files: Vec<ContractOutputSelection>,
    pub cache_path: PathBuf,
    pub src: PathBuf,
    pub test: PathBuf,
    pub script: PathBuf,
    pub out: PathBuf,
    pub libs: Vec<PathBuf>,
    pub remappings: Vec<RelativeRemapping>,
    pub auto_detect_remappings: bool,
    pub libraries: Vec<String>,
    pub cache: bool,
    pub build_info_path: Option<PathBuf>,
    pub force: bool,
    pub ephemeral: bool,
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
            cache_path: Default::default(),
            src: "src".into(),
            test: "test".into(),
            auto_detect_remappings: false,
            script: "script".into(),
            cache: true,
            libraries: Default::default(),
            out: "out".into(),
            remappings: Default::default(),
            force: false,
            ephemeral: true,
        }
    }
}

impl Solidity {
    pub fn new(
        root: String,
        _libs: Vec<String>,
        allow_paths: Vec<String>,
        include_paths: Vec<String>,
        src: PathBuf,
        test: PathBuf,
        auto_detect_remappings: bool,
        script: PathBuf,
        cache: bool,
        out: PathBuf,
        libraries: Vec<String>,
        force: bool,
        ephemeral: bool,
    ) -> Self {
        Self {
            root,
            libs: Default::default(),
            allow_paths,
            build_info_path: None,
            include_paths,
            extra_output: Default::default(),
            extra_output_files: Default::default(),
            cache_path: Default::default(),
            src,
            test,
            auto_detect_remappings,
            script,
            cache,
            libraries,
            out,
            remappings: Default::default(),
            force,
            ephemeral,
        }
    }

    // pub fn compile_with_foundry(&self) -> eyre::Result<ProjectCompileOutput> {
    //     let build_args = BuildArgs::default();
    //     // let mut config = build_args.try_load_config_emit_warnings()?;
    //     let mut config = Config::default();
    //     let mut project = config.project()?;

    //     let silent = false;

    //     // if install::install_missing_dependencies(&mut config, &project, self.args.silent)
    //     if install::install_missing_dependencies(&mut config, &project, silent)
    //         && config.auto_detect_remappings
    //     {
    //         // need to re-configure here to also catch additional remappings
    //         // config = self.load_config();
    //         project = config.project()?;
    //     }

    //     /*let skip = true;
    //     let filters = self.skip.unwrap_or_default();*/
    //     // if self.args.silent {
    //     if silent {
    //         compile::suppress_compile_with_filter(&project, Vec::new())
    //     } else {
    //         let compiler = ProjectCompiler::with_filter(false, false, Vec::new());
    //         compiler.compile(&project)
    //     }
    // }

    pub fn compile_with_foundry(&self) -> eyre::Result<ProjectCompileOutput> {
        let project_paths_args = ProjectPathsArgs {
            root: Some(PathBuf::from(&self.root)),
            ..Default::default()
        };

        let core_build_args = CoreBuildArgs {
            // TODO: remove force and use cached artifacts
            force: true,
            silent: false,
            project_paths: project_paths_args,
            ..Default::default()
        };

        let build_args = BuildArgs {
            args: core_build_args,
            ..Default::default()
        };
        build_args.run()
    }

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
            .cache(self.cache_path.join(SOLIDITY_FILES_CACHE_FILENAME))
            .sources(&self.src)
            .tests(&self.test)
            // .scripts(&self.script)
            .scripts(&self.root)
            .artifacts(&self.out)
            .libs(self.libs.clone())
            .remappings(self.get_all_remappings());

        if let Some(build_info_path) = &self.build_info_path {
            builder = builder.build_infos(build_info_path);
        }

        builder.build_with_root(&self.root)
    }

    pub fn project(&self) -> Result<Project, SolcError> {
        let mut project = Project::builder()
            .artifacts(self.artifacts())
            .paths(self.project_paths())
            .allowed_path(&self.root)
            .allowed_paths(&self.libs)
            .allowed_paths(&self.allow_paths)
            // .include_paths(&self.include_paths)
            // .solc_config(
            //     SolcConfig::builder()
            //         .settings(self.solc_settings()?)
            //         .build(),
            // )
            // .ignore_error_codes(self.ignored_error_codes.iter().copied().map(Into::into))
            // .set_compiler_severity_filter(if self.deny_warnings {
            //     Severity::Warning
            // } else {
            //     Severity::Error
            // })
            // .set_auto_detect(self.is_auto_detect())
            .set_auto_detect(true)
            // .set_offline(self.offline)
            // .set_cached(cached)
            .set_cached(true)
            // .set_build_info(cached & self.build_info)
            // .set_no_artifacts(no_artifacts)
            .set_build_info(true);

        if self.ephemeral {
            project = project.ephemeral().no_artifacts();
        }

        let project = project.build()?;

        if self.force {
            project.cleanup()?;
        }

        /*if let Some(solc) = self.ensure_solc()? {
            project.solc = solc;
        }*/

        Ok(project)
    }

    pub fn with_remappings(mut self, remappings: Vec<RelativeRemapping>) -> Self {
        self.remappings = remappings;
        self
    }

    pub fn with_path_root(mut self, root: PathBuf) -> Self {
        self.root = root.into_os_string().into_string().unwrap();
        self
    }
    pub fn with_root(mut self, root: String) -> Self {
        self.root = root;
        self
    }

    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }

    pub fn compile(&self) -> ProjectCompileOutput {
        let project = &self.project().unwrap();

        // dbg!(&project);

        // let mut project = Project::builder().build().unwrap();
        // project.paths.remappings = remappings;

        // let files = if path.is_dir() {
        //     self.get_sol_files(path)
        // } else if let Some(ext) = path.extension() {
        //     if ext == "sol" {
        //         vec![path]
        //     } else {
        //         panic!("Nothing valid to compile.");
        //     }
        // } else {
        //     panic!("Nothing valid to compile.");
        // };

        // let amount = files.len();
        // println!("Compiling {} files ...", amount);

        // let now = Instant::now();

        // let compiled = if auto_detect {
        //     // project.compile().unwrap()
        //     project.compile_files(files).unwrap()
        // } else {
        //     /*let sources = project.paths.read_sources().unwrap();
        //     project
        //         .compile_with_version(
        //             &Solc::find_svm_installed_version("0.8.0").unwrap().unwrap(),
        //             sources,
        //         )
        //         .unwrap()*/
        //     unimplemented!();
        // };

        // // project.rerun_if_sources_changed();

        // println!("Compiled in {}ms", now.elapsed().as_millis());

        // if compiled.has_compiler_errors() {
        //     let output = compiled.output();
        //     output.errors.iter().for_each(|error| {
        //         println!("{:#?}", error.formatted_message);
        //     });
        //     panic!();
        // }

        let compiled = project.compile().unwrap();

        // dbg!(&compiled);

        compiled
    }

    pub fn compile_artifacts(&self) -> BTreeMap<ArtifactId, ConfigurableContractArtifact> {
        let compiled = self.compile();

        compiled.into_artifacts().collect()
    }

    // get path of all .sol files
    pub fn get_sol_files(&self, path: PathBuf) -> Vec<PathBuf> {
        let mut files = Vec::new();

        self.visit_dirs(path.as_path(), &mut files)
            .expect("failed to get contracts");

        files
    }

    // could do caching, but explicitely excluding directory is probably good enough ?
    pub fn visit_dirs(&self, dir: &Path, files: &mut Vec<PathBuf>) -> eyre::Result<()> {
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
                        self.visit_dirs(&path, files)?;
                    }
                } else if self.is_sol_file(&path) {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    pub fn is_sol_file(&self, path: &Path) -> bool {
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
}
