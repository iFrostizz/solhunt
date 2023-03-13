use super::GasComparer;
use crate::{
    cmd::gas::MeteringData,
    solidity::{get_sol_files, version_from_source},
};
use ethers_solc::{compile::Solc, SolcVersion};
use indicatif::{ProgressBar, ProgressStyle};
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf};

/// walk sol files in the gas-metering folder and return a map to keep track of their name (finding id), version, in order to compile them and run the metering for each patch of solc
pub fn compile_metering() -> eyre::Result<(MeteringData, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gas-metering/contracts");

    let mut data = HashMap::new();

    let all_contracts = get_sol_files(root.clone());

    let all_sversions: Vec<_> = Solc::all_versions()
        .into_iter()
        .map(|ver| match ver {
            SolcVersion::Remote(ver) => ver,
            SolcVersion::Installed(ver) => ver,
        })
        .filter(|ver| ver.minor >= 5) // older ast is not supported
        .collect();

    let all_runs = all_contracts
        .iter()
        .map(|loc| {
            let mut file = File::open(loc.to_str().unwrap())?;
            let mut source = String::new();
            file.read_to_string(&mut source)?;

            let ver_req = version_from_source(source)?;

            Ok(all_sversions
                .iter()
                .filter(|ver| ver_req.matches(ver))
                .count())
        })
        .sum::<eyre::Result<usize>>()?;

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

    for location in all_contracts {
        let mut file = File::open(location.to_str().unwrap())?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;

        let ver_req = version_from_source(source)?;

        for ver in all_sversions.iter() {
            // compare for all matching versions
            if ver_req.matches(ver) {
                let mut gas_comparer = GasComparer::default()
                    .with_root(root.clone())
                    .with_location(location.clone())
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

                data.entry(folder_name)
                    .and_modify(
                        |f_v_g: &mut HashMap<String, HashMap<String, (String, String)>>| {
                            f_v_g
                                .entry(code.to_string())
                                .and_modify(|v_g| {
                                    v_g.insert(
                                        ver.clone().to_string(),
                                        (from.to_string(), to.to_string()),
                                    );
                                })
                                .or_default();
                        },
                    )
                    .or_default();

                bar.set_position(bar_pos);
                bar_pos += 1;
            }
        }
    }

    Ok((data, root))
}
