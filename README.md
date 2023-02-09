<img src="./images/solhunt.png" alt="Solhunt" align="right" width="120" height="120" />

## Solhunt

Solhunt is a Solidity static analyzer. It aims to be a tool used to find the most common vulnerabilities before starting a manual audit.

It has been built with modularity in mind. You can build new detection modules and build the binary again. Also, please open a PR with your custom detection modules !

Every detection module is made for one kind of vulnerability and raise its own custom type of findings. They are individually tested on small contracts (similar to unit tests) and on bigger ones, which have eventually caused a huge loss in the past (integration tests).

Tests aims to reduce the amount of false positives. It's easy to write detection modules, even if you are not particularly familiar with Rust. But here is a quick [guide](./src/modules/GUIDE.md).

This modularity will give you the choice on your way to proceed to find those bugs. One method would be "incremental analysis".

When doing an incremental analysis, you would create some profiles containing a range of detection modules, defined in a `foundry.toml`.

The first profile would find the easiest to fix (less false positives / negatives) issues. After mitigating those, you could go for the second profile with more false positives / negatives.

When you are confident about your codebase, you may use a gas optimization profile that would pack some common patterns in order to improve the gas savings without sacrificing the security of your smart contracts.

# Disclaimer

Remember that this is highly experimental software and that none of the proposed fixes are to be assumed correct.
Any changes that you make to your smart contract may introduce bugs, but if such a behaviour is noticed, please open an issue with a repro describing it.
