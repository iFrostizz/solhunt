use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "solhunt")]
#[clap(bin_name = "solhunt")]
pub struct Cmd {
    #[clap(value_name = "PATH")]
    pub path: String,
}