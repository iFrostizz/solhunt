# Solhunt

<img src="./images/solhunt.pn" alt="*your weapon of choice*" width="100" height="100"/>

This tool is a static analyzer currently shaped for Solidity. It aims to be your *hunting weapon of choice* to find the most common vulnerabilities before pushing them to production.
It has been built with modularity in mind. The core is the part that should be reusable and the most generic and the cli contains a bunch of detection modules.
Every detection module is made for one kind of vulnerability and raise its own custom type of findings. They are individually tested on small contracts (similar to unit tests) and on bigger ones, which have caused a huge loss in the past (integration tests).

This modularity will give you the choice on your way to proceed to find those bugs. One method would be "incremental analysis".
When doing an incremental analysis, you would create some profiles containing a range of detection modules, defined in a `foundry.toml` as an extra.
The first profile would find the easiest to fix (less false positives / negatives) issues. After mitigating those, you could go for the second profile which more false positives / negatives. When you are confident about your codebase, you may use a gas optimization profile that would pack a bunch of common patterns in order to improve the gas savings without sacrificing the security of your smart contracts.

# Disclaimer

Remember that this is highly experimental software and that none of the proposed fixes are 100% correct.
Any changes that you make to your smart contract may introduce bugs and don't forget to open an issue if such things happen, so that the detection modules can be improved.
