pub use cli::{Base64SubCommand, Opts, SubCommand};
pub use process::b64::{process_decode, process_encode};
pub use process::csv_convert::process_csv;
pub use process::gen_pass::process_genpass;

mod cli;
mod process;
