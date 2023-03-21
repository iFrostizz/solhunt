use super::parse::Analyze;
use crate::{
    formatter::Report,
    loader::get_all_visitors,
    solidity::{build_artifacts_source_maps, to_cached_artifacts, Solidity},
    walker::{Severity, Walker},
};
use ethers_solc::{artifacts::Optimizer, ArtifactId, ConfigurableContractArtifact};
use glob::{glob, GlobError};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

pub fn run_analysis(args: Analyze) -> eyre::Result<()> {
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

    let runs = args.optimizer_runs;
    let mut solidity = Solidity::default()
        .with_path_root(path.clone())
        .with_optimizer(Optimizer {
            enabled: Some(runs.is_some()),
            runs,
            details: None,
        })
        // .use_cache(false)
        .auto_remappings(true);

    let glob_path = path.join(args.glob);
    let glob_str = glob_path.to_str().unwrap();
    let glob = glob(glob_str)?.collect::<Result<HashSet<_>, GlobError>>()?;

    let compiled = solidity.compile().expect("Compilation failed");

    let artifacts = to_cached_artifacts(compiled.into_artifacts().collect())?;
    let artifacts: BTreeMap<ArtifactId, ConfigurableContractArtifact> = if path.is_dir() {
        artifacts
            .into_iter()
            .filter(|(id, _art)| glob.iter().any(|path| &id.source == path))
            .collect()
    } else {
        artifacts
    };

    if !artifacts.is_empty() {
        let source_map = build_artifacts_source_maps(&artifacts);

        let visitors = get_all_visitors();

        let mut walker = Walker::new(artifacts, source_map, visitors).with_bar(true);

        println!("Starting the analysis...");

        let findings = walker.traverse().expect("failed to traverse ast");
        let num_findings = findings.len();
        println!("Caught {num_findings} findings");

        if let Some(report_style) = args.style {
            let report = Report::new(report_style, path, findings, verbosity, args.github);
            report.format();
        }
    } else {
        println!("No artifacts matched the glob path");
    }

    Ok(())
}
