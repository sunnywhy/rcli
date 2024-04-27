use std::path::Path;

use clap::Parser;

use crate::cli::csv::CsvOpts;
use crate::cli::genpass::GenPassOpts;

pub use self::{
    base64::Base64Format, base64::Base64SubCommand, csv::OutputFormat, text::TextSignFormat,
    text::TextSubCommand,
};

mod base64;
mod csv;
mod genpass;
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
    #[command(subcommand)]
    Base64(Base64SubCommand),
    #[command(subcommand)]
    Text(TextSubCommand),
}

fn verify_file(filename: &str) -> anyhow::Result<String, &'static str> {
    // if filename is "-", it means read from stdin
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.into())
    } else {
        Err("Input file does not exist")
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
