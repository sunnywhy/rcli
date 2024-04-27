pub use cli::{Base64SubCommand, Opts, SubCommand, TextSubCommand};
pub use process::b64::{process_decode, process_encode};
pub use process::csv_convert::process_csv;
pub use process::gen_pass::process_genpass;
pub use process::text::{process_text_sign, process_text_verify};

mod cli;
mod process;
mod utils;
