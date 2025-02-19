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

use {
    anyhow::{anyhow, Context},
    std::{
        path::Path,
        process::{Command, ExitCode},
        str,
    },
};

mod cli;
pub(crate) use cli::Cli;

mod config;
pub(crate) use config::Config;

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

/// Get the MIME type of a file.
///
/// # Errors
///
/// Fails if the MIME type cannot be determined.
fn mime_type<P: AsRef<Path>>(file_path: P) -> anyhow::Result<String> {
    let output = Command::new("file")
        .arg("--brief")
        .arg("--mime-type")
        .arg(file_path.as_ref())
        .output()
        .context("Failed to execute 'file' command")?;

    if output.status.success() {
        let mime_type = str::from_utf8(&output.stdout).context("'file' output is invalid UTF-8")?;
        Ok(mime_type.trim().to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("{}", error).context("'file' command failed"))
    }
}
