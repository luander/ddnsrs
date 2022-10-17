//! Command Line Interface arguments and parser
use clap::Parser;

#[derive(Parser)]
#[clap(name = "DDNSrs")]
#[clap(author)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[clap(value_parser, env = "DDNSRS_HOSTNAME", help = "Hostname to be updated")]
    pub hostname: String,
}
