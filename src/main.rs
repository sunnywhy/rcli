// rcli csv -i input.csv -o output.json --header -d ','

use anyhow::Result;
use clap::Parser;

use rcli::{CmdExecutor, Opts};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    opts.cmd.execute().await
}
