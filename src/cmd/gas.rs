use super::parse::GasMetering;
use crate::interpreter::prepare::compile_metering;
use semver::Version;
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
    dbg!(&data);
    let mut path = root;
    path.pop();
    let path = path.join("metering.toml");

    let mut file = File::create(path)?;
    let toml = toml::to_string(&data)?;

    file.write_all(toml.as_bytes())?;

    Ok(())
}
