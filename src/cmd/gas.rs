use semver::{Version, VersionReq};

use super::parse::GasMetering;
use crate::interpreter::prepare::compile_metering;
use std::collections::hash_map::Entry;
use std::fs;
// use semver::Version;
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf};

/// HashMap that represents the gas metering database
/// module_name => finding_id => version => (from, to)
// pub type MeteringData = HashMap<String, HashMap<usize, HashMap<Version, (u64, u64)>>>;
pub type MeteringData = HashMap<String, HashMap<String, HashMap<String, (String, String)>>>;

/// For all metering contracts in the specific directory, compile it and run a metering on each of them.
/// The contract "From" will be compared to "To" with the function "gasMeter()" by default.
/// Can add decorators to custom the calldata. Version is parsed from the **single** pragma mentionned at the start
/// A lockfile will be written to in the TOML format to keep track of the gas changes
pub fn run_gas_metering(_args: GasMetering) -> eyre::Result<()> {
    // TODO: add some args parsing logic

    let (data, root) = compile_metering()?;

    write_to_base(root, data)?;

    Ok(())
}

pub fn write_to_base(root: PathBuf, data: MeteringData) -> eyre::Result<()> {
    let mut path = root;
    path.pop();
    let path = path.join("metering.toml");

    let mut file = File::create(path)?;
    let toml = toml::to_string(&data)?;

    file.write_all(toml.as_bytes())?;

    Ok(())
}

/// return the biggest gas saved (if any) for a module code satisfying a version
pub fn get_gas_diff(module: String, code: usize, ver_req: VersionReq) -> Option<u64> {
    let file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gas-metering/contracts/metering.toml");

    if file.exists() {
        let mut content = String::new();
        let mut file = fs::File::open(file).unwrap();
        file.read_to_string(&mut content).unwrap();

        let mut data: MeteringData = match toml::from_str(&content) {
            Ok(d) => d,
            Err(_) => {
                return None;
            }
        };

        match data.entry(module) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();

                match entry.entry(code.to_string()) {
                    Entry::Occupied(entry) => entry
                        .get()
                        .iter()
                        .map(|(v, g)| {
                            let version = Version::parse(v).unwrap();
                            let (gf, gt) =
                                (g.0.parse::<u64>().unwrap(), g.1.parse::<u64>().unwrap());

                            if ver_req.matches(&version) {
                                gf.saturating_sub(gt)
                            } else {
                                0
                            }
                        })
                        .max(),
                    Entry::Vacant(_) => None,
                }
            }
            Entry::Vacant(_) => None,
        }
    } else {
        None
    }
}
