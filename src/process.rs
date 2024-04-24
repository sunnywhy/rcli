use std::fs;

use anyhow::Result;
use csv::Reader;
use serde_json::Value;

use crate::opts::OutputFormat;

pub fn process_csv(input: &str, output: String, format: OutputFormat) -> Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for record in reader.records() {
        let record = record?;
        let value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(value);
    }

    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
    };
    fs::write(output, content)?;

    Ok(())
}
