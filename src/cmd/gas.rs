use super::parse::GasMetering;
use crate::interpreter::prepare::compile_metering;

/// For all metering contracts in the specific directory, compile it and run a metering on each of them.
/// The contract "From" will be compared to "To" with the function "gasMeter()" by default.
/// Can add decorators to custom the calldata. Version is parsed from the **single** pragma mentionned at the start
/// A lockfile will be written to in the TOML format to keep track of the gas changes
pub fn run_gas_metering(args: GasMetering) {
    compile_metering();
}
