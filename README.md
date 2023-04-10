<img src="./images/solhunt.png" alt="Solhunt" align="right" width="120" height="120" />

# Solhunt

## Intro

Solhunt is a Solidity static analyzer. It aims to be a tool used to find the most common vulnerabilities before starting a manual audit.

It has been built with modularity in mind. You can build new detection modules and build the binary again. Also, please open a PR with your custom detection modules !

Every detection module is made for one kind of vulnerability and raise its own custom type of findings. They are individually tested on small contracts (similar to unit tests) and on bigger ones, which have eventually caused a huge loss in the past (integration tests).

Tests aims to reduce the amount of false positives. It's easy to write detection modules, even if you are not particularly familiar with Rust. But here is a quick [guide](./src/modules/GUIDE.md).

## Usage

### Basic command

```sh
Usage: solhunt <COMMAND>

Commands:
  analyze  Run a static analysis
  gas      Launch gas meterings
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Analyze subcommand

```sh
Usage: solhunt analyze [OPTIONS] [PATH] [NAME]

Arguments:
  [PATH]  root of the project [default: .]
  [NAME]

Options:
  -m, --modules <MODULES>
          Include only these modules
  -e, --except-modules <EXCEPT_MODULES>
          Exclude these modules
  -v, --verbosity <VERBOSITY>
          Verbosity of the findings
  -s, --style <STYLE>
          Style of the report [possible values: list, cmd, md, html]
  -o, --optimizer-runs <OPTIMIZER_RUNS>
          specifiy the optimizer runs
  -g, --glob <GLOB>
          glob path for artifacts to analyze [default: {src,contracts}/**/*.sol]
      --github <GITHUB>
          base location of github path [aliases: gh]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Gas metering subcommand

```sh
Usage: solhunt gas [OPTIONS]

Options:
  -e, --except-modules <EXCEPT_MODULES>  Exclude these modules
  -p, --path <PATH>                      Location of files to meter
  -r, --reset                            reset the metering database
  -h, --help                             Print help
  -V, --version                          Print version
```

## Building

### Install the binary

#### From git

```sh
cargo install --git https://github.com/iFrostizz/Solhunt
```

#### From path

cd into the project root folder and

```sh
cargo install --path .
```

## Disclaimer

Remember that this is highly experimental software and that none of the proposed fixes are to be assumed correct.
Any changes that you make to your smart contract may introduce bugs, but if such a behaviour is noticed, please open an issue with a repro describing it.
