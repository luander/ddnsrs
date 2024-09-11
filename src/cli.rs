//! Command Line Interface arguments and parser
use clap::Parser;

#[derive(Parser, Clone)]
#[clap(name = "DDNSrs")]
#[clap(author)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[command(version, about)]
pub struct Cli {
    #[clap(value_parser, env = "ddnsrs_hostname", help = "hostname to be updated")]
    pub hostname: String,

    #[clap(
        value_parser,
        env = "DDNSRS_CLOUDFLARE_KEY",
        help = "Cloudflare API Key"
    )]
    pub cloudflare_key: String,

    #[clap(
        value_parser,
        env = "DDNSRS_INTERVAL",
        help = "how often updates will happen, in seconds",
        default_value_t = 300
    )]
    pub interval: u32,
}
