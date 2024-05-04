use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::{process_jwt_sign, process_jwt_verify, CmdExecutor};

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(about = "generate a new token, exp supports s/m/h/d suffixes")]
    Sign(JwtSignOpts),
    #[command(about = "verify a token")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(long)]
    pub sub: String,
    #[arg(long)]
    pub aud: String,
    #[arg(long, value_parser = parse_exp, default_value = "1d")]
    pub exp: usize,
    #[arg(long, default_value = "thisisasecret")]
    pub secret: String,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long)]
    pub token: String,
    #[arg(long, default_value = "thisisasecret")]
    pub secret: String,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let token = process_jwt_sign(&self.sub, &self.aud, self.exp, &self.secret)?;
        println!("{}", token);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verified = process_jwt_verify(&self.token, &self.secret);
        println!("{}", verified);
        Ok(())
    }
}

fn parse_exp(s: &str) -> anyhow::Result<usize> {
    let re = regex::Regex::new(r"^(\d+)([smhd])$")?;
    let caps = re
        .captures(s)
        .ok_or_else(|| anyhow::anyhow!("invalid duration"))?;
    let num = caps
        .get(1)
        .map(|m| m.as_str().parse::<usize>())
        .ok_or_else(|| anyhow::anyhow!("invalid duration"))??;
    let unit = caps
        .get(2)
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("invalid duration"))?;

    let duration = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 60 * 60,
        "d" => num * 60 * 60 * 24,
        _ => anyhow::bail!("invalid duration unit"),
    };
    let now = chrono::Utc::now().timestamp() as usize;
    Ok(now + duration)
}
