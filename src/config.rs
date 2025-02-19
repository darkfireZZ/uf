use {
    anyhow::{anyhow, bail, Context},
    std::{
        env,
        ffi::OsString,
        fs::File,
        io::{self, BufRead, BufReader},
        path::{Path, PathBuf},
    },
};

/// Get the home directory of the current user.
///
/// # Errors
///
/// Fails if the `HOME` environment variable cannot be read or is not set.
fn home_dir() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .context("HOME environment variable not set")
        .map(PathBuf::from)
}

/// Get the path to the configuration file.
///
/// # Errors
///
/// Fails if the home directory of the current user cannot be determined.
fn config_path() -> anyhow::Result<PathBuf> {
    home_dir().map(|mut home_dir| {
        home_dir.push(".config/uf.conf");
        home_dir
    })
}

/// Configuration.
#[derive(Debug)]
pub struct Config {
    mappings: Vec<Mapping>,
}

#[derive(Debug)]
enum Mapping {
    Extension {
        extension: OsString,
        program: String,
    },
    Mime {
        mime: String,
        program: String,
    },
}

impl Config {
    /// Load the configuration file.
    ///
    /// # Errors
    ///
    /// Fails in any of the following cases:
    /// - The location of the configuration file cannot be determined.
    /// - The configuration file cannot be opened.
    /// - The configuration file cannot be read.
    /// - The configuration file is invalid.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = config_path()?;
        let config_file = File::open(&config_path).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => {
                anyhow!("Config file not found: {}", config_path.display())
            }
            _ => anyhow::Error::new(error).context(format!(
                "Failed to open config file: {}",
                config_path.display()
            )),
        })?;
        let config_reader = BufReader::new(config_file);

        Self::parse(config_reader)
    }

    /// Parse the configuration file.
    ///
    /// # Errors
    ///
    /// Fails in any of the following cases:
    /// - The configuration file cannot be read.
    /// - The configuration file is invalid.
    fn parse<R: BufRead>(reader: R) -> anyhow::Result<Self> {
        let mappings = reader
            .lines()
            .enumerate()
            .map(|(line_index, line)| (line_index + 1, line))
            .map(|(line_number, line)| {
                let line = line.with_context(|| format!("Failed to read line {}", line_number))?;
                let line = line
                    .split_once('#')
                    .map_or(line.as_str(), |(line, _)| line)
                    .trim();
                if line.is_empty() {
                    return Ok(None);
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() != 3 {
                    bail!("Invalid line: {}", line);
                }
                let program = parts[2].to_string();
                match parts[0] {
                    "ext" => Ok(Some(Mapping::Extension {
                        extension: OsString::from(parts[1]),
                        program,
                    })),
                    "mime" => Ok(Some(Mapping::Mime {
                        mime: parts[1].to_string(),
                        program,
                    })),
                    _ => bail!("Invalid line: {}", line),
                }
            })
            .filter_map(|result| result.transpose())
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to parse config file")?;

        Ok(Self { mappings })
    }

    /// Get the program configured for opening a file.
    ///
    /// # Errors
    ///
    /// Fails in any of the following cases:
    /// - The MIME type of the file cannot be determined.
    /// - No program is configured for the file.
    pub fn get_program<P: AsRef<Path>>(&self, file_path: P) -> anyhow::Result<&str> {
        let extension = file_path.as_ref().extension();
        let mime = crate::mime_type(&file_path)?;

        self.mappings
            .iter()
            .find_map(|mapping| match mapping {
                Mapping::Extension {
                    extension: map_extension,
                    program,
                } if Some(map_extension.as_os_str()) == extension => Some(program.as_str()),
                Mapping::Mime {
                    mime: map_mime,
                    program,
                } if *map_mime == mime => Some(program.as_str()),
                _ => None,
            })
            .ok_or_else(|| match extension {
                Some(extension) => anyhow!(
                    "No program found for MIME type '{mime}', extension '{}'",
                    extension.to_string_lossy()
                ),
                None => anyhow!("No program found for MIME type '{mime}'"),
            })
    }
}
