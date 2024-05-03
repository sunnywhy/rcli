pub use cli::{Base64SubCommand, HttpSubCommand, Opts, SubCommand, TextSignFormat, TextSubCommand};
pub use process::b64::{process_decode, process_encode};
pub use process::csv_convert::process_csv;
pub use process::gen_pass::process_genpass;
pub use process::http_serve::process_http_serve;
pub use process::text::{process_generate_key, process_text_sign, process_text_verify};

mod cli;
mod process;
mod utils;

#[allow(async_fn_in_trait)]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
