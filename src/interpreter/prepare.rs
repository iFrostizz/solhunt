use super::GasComparer;
use crate::solidity::{get_sol_files, version_from_source};
use ethers_solc::{compile::Solc, SolcVersion};
use std::path::PathBuf;
use std::{fs::File, io::prelude::*};

/// walk sol files in the gas-metering folder and return a map to keep track of their name (finding id), version, in order to compile them and run the metering for each patch of solc
pub fn compile_metering() -> eyre::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gas-metering/contracts");

    let all_contracts = get_sol_files(root.clone());

    let all_sversions: Vec<_> = Solc::all_versions()
        .into_iter()
        .map(|ver| match ver {
            SolcVersion::Remote(ver) => ver,
            SolcVersion::Installed(ver) => ver,
        })
        .filter(|ver| ver.minor >= 5) // older ast is not supported
        .collect();

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

                println!("{} {} {} {:?}", location.display(), from, to, ver.clone());
            }
        }
    }

    Ok(())
}
