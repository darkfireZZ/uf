#![warn(
    clippy::pedantic,
    clippy::absolute_paths,
    clippy::allow_attributes_without_reason,
    clippy::dbg_macro,
    clippy::exit,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    missing_debug_implementations,
    missing_docs
)]
// The following lints are enable by default in clippy::pedantic, but are disabled here because
// they are too aggressive.
#![allow(clippy::module_name_repetitions, reason = "Occasionally useful")]
#![allow(clippy::too_many_lines, reason = "This is not bad in my opinion")]

use std::process::ExitCode;

mod cli;
pub(crate) use cli::Cli;

mod config;
pub(crate) use config::Config;

mod mime;
pub(crate) use mime::MimeType;

/// Run the application.
#[must_use]
pub fn run() -> ExitCode {
    let args = Cli::parse();
    match args.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
