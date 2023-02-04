use clap::Parser;
use ethers_solc::remappings::{RelativeRemapping, Remapping};

use serde::Serialize;
use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use crate::walker::Severity;

#[derive(Parser, Debug, Serialize)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "solhunt")]
#[clap(bin_name = "solhunt")]
pub struct Cmd {
    #[clap(value_name = "PATH", default_value = ".")]
    pub path: String,
    #[clap(short, long, help = "Include only these modules")]
    pub modules: Option<Vec<String>>,
    #[clap(short, long, help = "Exclude these modules")]
    pub except_modules: Option<Vec<String>>,
    // TODO: use "hmgi" instead and let max verbosity by default
    // TODO: allow configuring of ignored directories through a .toml file
    // source, foundry: foundry/common/src/evm.rs
    /// Verbosity
    ///
    /// Pass multiple times to increase the verbosity (e.g. -v, -vv, -vvv).
    ///
    /// Verbosity levels:
    /// - 0: Print only High / Medium threats
    /// - 1: Also print Low
    /// - 2: Also print Gas
    /// - 3: Also print informal / code style
    /// - 4: Print tracing logs
    // #[clap(long, short, parse(from_occurrences), verbatim_doc_comment)]
    // #[serde(skip)]
    // pub verbosity: u8,
    pub verbosity: Option<String>,
}

pub fn get_working_path(add_path: String) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(current_dir().expect("could not get current path"));
    path.push(add_path);

    path.canonicalize().expect("Invalid path")
}

pub fn parse() -> (PathBuf, Vec<Severity>) {
    let args = Cmd::parse();

    // TODO: filter based on rust module name
    // let all_modules = get_all_modules(); // get em' all before loading only those that we want

    // // Only those specified
    // let modules: Vec<DynModule> = match args.modules {
    //     Some(modules_names) => all_modules
    //         .into_iter()
    //         .filter(|module| modules_names.contains(&module.name))
    //         .collect(),
    //     None => all_modules, // don't touch if not specified
    // };

    // // Remove those we don't want
    // let modules: Vec<DynModule> = match args.except_modules {
    //     Some(modules_names) => modules
    //         .into_iter()
    //         .filter(|module| !modules_names.contains(&module.name))
    //         .collect(),

    //     None => modules,
    // };

    let verbosity = if let Some(args_verb) = args.verbosity {
        let mut severities = HashMap::from([
            ('h', Severity::High),
            ('m', Severity::Medium),
            ('l', Severity::Low),
            ('g', Severity::Gas),
            ('i', Severity::Informal),
        ]);

        args_verb
            .chars()
            .filter_map(|c| severities.remove(&c))
            .collect()
    } else {
        vec![
            Severity::Informal,
            Severity::Gas,
            Severity::Medium,
            Severity::High,
        ]
    };

    (get_working_path(args.path), verbosity)
}

pub fn get_remappings(path: &Path) -> Vec<RelativeRemapping> {
    let base_path = path.to_path_buf();
    let mut remappings: Vec<RelativeRemapping> = Vec::new();

    let remappings_file = base_path.join("remappings.txt");
    if remappings_file.is_file() {
        let content = fs::read_to_string(remappings_file)
            .map_err(|err| err.to_string())
            .unwrap();

        let rem_lines = content.split('\n').collect::<Vec<&str>>();
        let rem = rem_lines
            .iter()
            .filter(|l| l != &&"")
            .map(|l| l.split_once('='))
            .collect::<Vec<Option<(&str, &str)>>>();
        rem.iter().for_each(|pair| {
            if let Some((lib, path)) = pair {
                let full_path = base_path.join(path);
                remappings.push(
                    Remapping {
                        name: lib.to_string(),
                        path: full_path.into_os_string().into_string().unwrap(),
                    }
                    .into_relative(base_path.clone()),
                );
            }
        });
    }

    remappings
}
