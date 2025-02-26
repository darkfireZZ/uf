use {
    anyhow::{anyhow, Context},
    std::{
        fmt::{self, Display, Formatter},
        path::Path,
        process::Command,
        str,
    },
};

#[derive(Debug)]
pub struct MimeType {
    supertype: String,
    subtype: String,
}

impl MimeType {
    /// Get the MIME type of a file.
    ///
    /// # Errors
    ///
    /// Fails if the MIME type cannot be determined.
    pub fn detect<P: AsRef<Path>>(file_path: P) -> anyhow::Result<Self> {
        let output = Command::new("file")
            .arg("--brief")
            .arg("--dereference")
            .arg("--mime-type")
            .arg(file_path.as_ref())
            .output()
            .context("Failed to execute 'file' command")?;

        if output.status.success() {
            let mime_type =
                str::from_utf8(&output.stdout).context("'file' output is invalid UTF-8")?;
            let (supertype, subtype) = mime_type
                .trim()
                .split_once('/')
                .with_context(|| format!("'file' output is not a valid MIME type: {mime_type}"))?;
            Ok(Self {
                supertype: supertype.to_string(),
                subtype: subtype.to_string(),
            })
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("{}", error).context("'file' command failed"))
        }
    }

    pub fn supertype(&self) -> &str {
        &self.supertype
    }

    pub fn subtype(&self) -> &str {
        &self.subtype
    }
}

impl Display for MimeType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.supertype, self.subtype)
    }
}
