# Gas metering

## Description

Folder containing contracts and scripts for the gas metering feature.
Also has a lock file to keep track of the gas usage.
Gas metering is done for every versions of Solidity in order to give reliable results on the gas reports, and to not flag false positive that may be fixed in further versions of Solidity.

## TODO

file cache to not re-run a metering on the same contract
