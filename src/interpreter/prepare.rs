use super::GasComparer;
use crate::{
    cmd::{bars::get_bar, gas::MeteringData},
    solidity::{equi_ver, get_sol_files, version_from_source, Solidity},
};
use ethers_solc::{compile::Solc, ArtifactId, ConfigurableContractArtifact, SolcVersion};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use semver::Version;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// walk sol files in the gas-metering folder and return a map to keep track of their name (finding id), version, in order to compile them and run the metering for each patch of solc
pub fn compile_metering(
    root: &PathBuf,
    data: MeteringData,
    path: &Path, // represents the path of the only contracts we wil meter
) -> eyre::Result<MeteringData> {
    let data = Arc::new(Mutex::new(data));

    let all_contracts = get_sol_files(path);

    // list all svm versions, wether installed or not
    let all_sversions: Vec<_> = Solc::all_versions()
        .into_iter()
        .map(|ver| match ver {
            SolcVersion::Remote(ver) => ver,
            SolcVersion::Installed(ver) => ver,
        })
        .filter(|ver| ver.minor >= 5) // older ast is not supported
        .collect();

    // map of the location of each contract per their compatible solc version
    let mut versioned_locations: HashMap<Version, Vec<PathBuf>> = HashMap::new();

    for loc in all_contracts.iter() {
        let mut file = File::open(loc.to_str().unwrap())?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        let ver_req = version_from_source(source)?;

        all_sversions.clone().into_iter().for_each(|ver| {
            if ver_req.matches(&ver) {
                let entry = versioned_locations.entry(ver).or_default();
                entry.push(loc.to_path_buf());
            }
        });
    }

    let spinner = ProgressBar::new_spinner();

    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
    );
    spinner.set_message("Compiling...");

    let versioned_artifacts = versioned_locations
        .par_iter()
        .map(|(ver, files)| {
            let mut solidity = Solidity::default()
                .with_path_root(root.clone())
                .with_locations(files.to_vec())
                .use_cache(false) // cache is buggy with this configuration
                .silent()
                .with_version(ver.clone())
                .unwrap();

            let artifacts = solidity
                .compile_artifacts()
                .expect("failed to compile artifacts");

            for id in artifacts.keys() {
                assert!(equi_ver(&id.version, ver));
            }

            (ver.clone(), artifacts)
        })
        .collect::<HashMap<Version, BTreeMap<ArtifactId, ConfigurableContractArtifact>>>();

    spinner.finish_with_message("Done compiling");

    let len = versioned_locations
        .values()
        .map(|l| (l.len() * 2) as u64)
        .sum::<u64>();
    let message = String::from("Running gas meterings...");

    let bar = get_bar(len, message);

    versioned_artifacts
        .into_par_iter()
        .for_each(|(ver, artifacts)| {
            let artifacts_locations: Vec<PathBuf> =
                artifacts.keys().map(|id| id.source.clone()).collect();

            for location in artifacts_locations.iter() {
                let from_to_artifacts_iter =
                    artifacts.iter().filter(|(id, _)| &id.source == location);

                // should only find two artifacts, on "from" and one "to"
                assert_eq!(from_to_artifacts_iter.clone().count(), 2);

                let mut art_from = None;
                let mut art_to = None;

                from_to_artifacts_iter.for_each(|(id, artifact)| {
                    if id.name == "From" {
                        art_from = Some(BTreeMap::from([(id.clone(), artifact.clone())]));
                    } else if id.name == "To" {
                        art_to = Some(BTreeMap::from([(id.clone(), artifact.clone())]));
                    }
                });

                // let art_from = art_from.ok_or(eyre::eyre!("No `From` artifact"))?;
                // let art_to = art_to.ok_or(eyre::eyre!("No `To` artifact"))?;
                let art_from = art_from.expect("No `From` artifact");
                let art_to = art_to.expect("No `To` artifact");

                let mut gas_comparer = GasComparer::default()
                    .with_root(root.clone())
                    .with_location(location.clone())
                    .with_artifacts((art_from, art_to))
                    .with_version(ver.clone());

                let (from, to) = match gas_comparer.run() {
                    Ok(a) => a,
                    Err(err) => {
                        let mini_path = location.strip_prefix(root).unwrap();
                        panic!("err for location `{}`: `{err}`", mini_path.display());
                    }
                };

                let file_stem = location
                    .file_stem()
                    .expect("could not get file name")
                    .to_os_string()
                    .into_string()
                    .unwrap();

                let code: usize = file_stem
                    .parse()
                    .expect("should be named `code.sol`, got {file_stem}");

                let folder_name = location
                    .parent()
                    .expect("could not get parent")
                    .file_name()
                    .expect("could not get file name")
                    .to_str()
                    .unwrap()
                    .to_string();

                let mut d = data.lock().unwrap();

                let d1: &mut HashMap<String, HashMap<String, (String, String)>> =
                    d.entry(folder_name).or_default();
                let d2 = d1.entry(code.to_string()).or_default();
                d2.insert(ver.clone().to_string(), (from.to_string(), to.to_string()));

                bar.inc(1);
            }
        });

    let data = Arc::try_unwrap(data).unwrap().into_inner().unwrap();

    Ok(data)
}
