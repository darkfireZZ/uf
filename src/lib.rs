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
pub fn run() -> ExitCode {
    let args = Cli::parse();
    match args.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {:#}", error);
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
