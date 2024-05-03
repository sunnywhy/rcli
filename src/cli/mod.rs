use std::path::{Path, PathBuf};

use clap::Parser;

use crate::cli::csv::CsvOpts;
use crate::cli::genpass::GenPassOpts;
use crate::CmdExecutor;

pub use self::{
    base64::Base64Format, base64::Base64SubCommand, csv::OutputFormat, http::HttpSubCommand,
    text::TextSignFormat, text::TextSubCommand,
};

mod base64;
mod csv;
mod genpass;
mod http;
mod text;

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV or convert CSV to other formats")]
    Csv(CsvOpts),
    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
    #[command(subcommand, about = "Base64 encode/decode")]
    Base64(Base64SubCommand),
    #[command(subcommand, about = "Text sign/verify")]
    Text(TextSubCommand),
    #[command(subcommand, about = "HTTP server")]
    Http(HttpSubCommand),
}

impl CmdExecutor for SubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            SubCommand::Csv(opts) => opts.execute().await,
            SubCommand::GenPass(opts) => opts.execute().await,
            SubCommand::Base64(subcmd) => subcmd.execute().await,
            SubCommand::Text(subcmd) => subcmd.execute().await,
            SubCommand::Http(subcmd) => subcmd.execute().await,
        }
    }
}

fn verify_file(filename: &str) -> anyhow::Result<String, &'static str> {
    // if filename is "-", it means read from stdin
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("Input file does not exist")
    }
}

fn verify_path(path: &str) -> anyhow::Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if Path::new(path).exists() && p.is_dir() {
        Ok(path.into())
    } else {
        Err("Path does not exist or is not a directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("*"), Err("Input file does not exist"));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_file("nonexistent"), Err("Input file does not exist"));
    }
}
