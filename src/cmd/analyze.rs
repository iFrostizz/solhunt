use ethers_solc::artifacts::Optimizer;

use super::parse::Analyze;
use crate::{
    formatter::Report,
    loader::get_all_visitors,
    solidity::{build_source_maps, Solidity},
    walker::{Severity, Walker},
};
use std::{collections::HashMap, path::PathBuf};

pub fn run_analysis(args: Analyze) {
    let mut severities = HashMap::from([
        ('h', Severity::High),
        ('m', Severity::Medium),
        ('l', Severity::Low),
        ('g', Severity::Gas),
        ('i', Severity::Informal),
    ]);

    let verbosity: Vec<Severity> = if let Some(args_verb) = args.verbosity {
        args_verb
            .chars()
            .filter_map(|c| severities.remove(&c))
            .collect()
    } else {
        severities.values().map(|s| s.to_owned()).collect()
    };

    let path = PathBuf::from(args.path).canonicalize().unwrap();
    let report_style = args.style;

    let runs = args.optimizer_runs;

    let mut solidity = Solidity::default()
        .with_path_root(path.clone())
        .with_cache_path(path.join("cache"))
        .with_optimizer(Optimizer {
            enabled: if runs.is_some() { Some(true) } else { None },
            runs,
            details: None,
        })
        .auto_remappings(true);

    let compiled = solidity.compile().expect("Compilation failed");
    let output = compiled.clone().output();

    let source_map = build_source_maps(output);

    // TODO: configurable with glob
    let _included_folders: Vec<String> = vec![String::from("src")];

    let artifacts = compiled
        .into_artifacts()
        .filter(|(id, _art)| {
            let root_path = &path;
            if root_path.is_dir() {
                // only filter if not "file-only"
                let abs_path = &id.source;
                match abs_path.strip_prefix(root_path) {
                    // TODO: tracing this
                    // panic!(
                    //     "Failed to strip root path: `{}` from `{}`, {}",
                    //     root_path.to_string_lossy(),
                    //     abs_path.to_string_lossy(),
                    //     e
                    // )
                    Ok(_other_path) => {
                        // let first_folder = other_path
                        //     .iter()
                        //     .next()
                        //     .expect("Failed to get first folder");
                        // // only take included folders
                        // included_folders.contains(&first_folder.to_string_lossy().to_string())
                        true
                    }
                    // No need to take care of artifacts outside of the project root
                    // they are usually libraries
                    _ => false,
                }
            } else {
                false
            }
        })
        .collect();

    let visitors = get_all_visitors();

    let mut walker = Walker::new(artifacts, source_map, visitors, path.clone());

    println!("Starting the analysis...");

    let findings = walker.traverse().expect("failed to traverse ast");
    let num_findings = findings.len();
    println!("Caught {num_findings} findings");

    let report = Report::new(report_style, path, findings, verbosity);
    report.format();
}
