use super::GasComparer;
use crate::{
    cmd::gas::MeteringData,
    solidity::{get_sol_files, version_from_source, Solidity},
};
use ethers_solc::{compile::Solc, ArtifactId, ConfigurableContractArtifact, SolcVersion};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use semver::Version;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::prelude::*,
    path::PathBuf,
};

// TODO: compile the whole folder only once (use cache), and pass all the artifacts
// compile as much files at once with the same version as we can
/// walk sol files in the gas-metering folder and return a map to keep track of their name (finding id), version, in order to compile them and run the metering for each patch of solc
pub fn compile_metering() -> eyre::Result<(MeteringData, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gas-metering/contracts");

    let mut data = HashMap::new();

    let all_contracts = get_sol_files(root.clone());

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
    let mut contract_versions: HashMap<Version, Vec<PathBuf>> = HashMap::new();

    let all_runs = all_contracts
        .iter()
        .map(|loc| {
            let mut file = File::open(loc.to_str().unwrap())?;
            let mut source = String::new();
            file.read_to_string(&mut source)?;

            let ver_req = version_from_source(source)?;

            Ok(all_sversions
                .clone()
                .into_iter()
                .filter(|ver| {
                    if ver_req.matches(ver) {
                        let locs = contract_versions.entry(ver.clone()).or_default();
                        locs.push(loc.to_path_buf());
                        true
                    } else {
                        false
                    }
                })
                .count())
        })
        .sum::<eyre::Result<usize>>()?;

    // TODO: write compilation time
    let versioned_artifacts = contract_versions
        .par_iter()
        .map(|(ver, files)| {
            let mut solidity = Solidity::default()
                .with_path_root(root.clone())
                .with_locations(files.to_vec())
                .silent()
                .with_version(ver.clone())
                .unwrap();

            (ver.clone(), solidity.compile_artifacts().unwrap())
        })
        .collect::<HashMap<Version, BTreeMap<ArtifactId, ConfigurableContractArtifact>>>();

    let bar = ProgressBar::new(all_runs as u64);

    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {spinner:.blue} [{elapsed_precise}] {bar:100.cyan/blue} [{human_pos}/{human_len}]",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    bar.set_message("Running gas meterings...");

    let mut bar_pos = 1;

    for (ver, artifacts) in versioned_artifacts.into_iter() {
        let artifacts_locations: Vec<PathBuf> =
            artifacts.keys().map(|id| id.source.clone()).collect();

        for location in artifacts_locations.iter() {
            let from_to_artifacts_iter = artifacts.iter().filter(|(id, _)| &id.source == location);

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

            let art_from = art_from.ok_or(eyre::eyre!("No `From` artifact"))?;
            let art_to = art_to.ok_or(eyre::eyre!("No `To` artifact"))?;

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
                .ok_or(eyre::eyre!("couldn't get file name for {:#?}", location))?
                .to_os_string()
                .into_string()
                .unwrap();

            let code: usize = file_stem
                .parse()
                .expect("should be named `code.sol`, got {file_stem}");

            let folder_name = location
                .parent()
                .ok_or(eyre::eyre!("couldn't get parent for {:#?}", location))?
                .file_name()
                .ok_or(eyre::eyre!("couldn't get file name for {:#?}", location))?
                .to_str()
                .unwrap()
                .to_string();

            let d1: &mut HashMap<String, HashMap<String, (String, String)>> =
                data.entry(folder_name).or_default();
            let d2 = d1.entry(code.to_string()).or_default();
            d2.insert(ver.clone().to_string(), (from.to_string(), to.to_string()));

            bar.set_position(bar_pos);
            bar_pos += 1;
        }
    }

    Ok((data, root))
}
