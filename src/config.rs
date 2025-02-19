use {
    crate::MimeType,
    anyhow::{anyhow, bail, Context},
    std::{
        env,
        ffi::{OsStr, OsString},
        fs::File,
        io::{self, BufRead, BufReader},
        path::{Path, PathBuf},
        str::FromStr,
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
        mime: MimeTypeKey,
        program: String,
    },
}

impl Mapping {
    /// Return the program to use if the mapping matches the file, or `None` if it does not.
    fn get_program(&self, mime: &MimeType, extension: Option<&OsStr>) -> Option<&str> {
        match self {
            Self::Extension {
                extension: map_extension,
                program,
            } if Some(map_extension.as_os_str()) == extension => Some(program),
            Self::Mime {
                mime: map_mime,
                program,
            } if map_mime.matches(mime) => Some(program),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct MimeTypeKey {
    supertype: String,
    subtype: MimeSubtypeKey,
}

impl MimeTypeKey {
    /// Return whether a MIME type matches this key.
    pub fn matches(&self, mime: &MimeType) -> bool {
        self.supertype.eq_ignore_ascii_case(mime.supertype())
            && match &self.subtype {
                MimeSubtypeKey::Specific(subtype) => subtype.eq_ignore_ascii_case(mime.subtype()),
                MimeSubtypeKey::Wildcard => true,
            }
    }
}

impl FromStr for MimeTypeKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = || anyhow!("Invalid MIME type: {}", s);
        let char_allowed =
            |c: char| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.' || c == '_';
        let (supertype, subtype) = s.split_once('/').ok_or_else(error)?;
        if supertype.is_empty() || subtype.is_empty() || !supertype.chars().all(char_allowed) {
            return Err(error());
        }
        let subtype = match subtype {
            "*" => MimeSubtypeKey::Wildcard,
            subtype => {
                if !subtype.chars().all(char_allowed) {
                    return Err(error());
                }
                MimeSubtypeKey::Specific(subtype.to_string())
            }
        };
        Ok(Self {
            supertype: supertype.to_string(),
            subtype,
        })
    }
}

#[derive(Debug)]
enum MimeSubtypeKey {
    Specific(String),
    Wildcard,
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
    #[allow(
        clippy::match_on_vec_items,
        reason = "Using get() instead of [] would be unnecessarily verbose."
    )]
    fn parse<R: BufRead>(reader: R) -> anyhow::Result<Self> {
        let mappings = reader
            .lines()
            .enumerate()
            .map(|(line_index, line)| (line_index + 1, line))
            .map(|(line_number, line)| {
                let line = line.with_context(|| format!("Failed to read line {line_number}"))?;
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
                        mime: parts[1]
                            .parse()
                            .with_context(|| format!("Invalid line: {line}"))?,
                        program,
                    })),
                    _ => bail!("Invalid line: {}", line),
                }
            })
            .filter_map(Result::transpose)
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
        let mime = MimeType::detect(&file_path)?;

        self.mappings
            .iter()
            .find_map(|mapping| mapping.get_program(&mime, extension))
            .ok_or_else(|| match extension {
                Some(extension) => anyhow!(
                    "No program found for MIME type '{mime}', extension '{}'",
                    extension.to_string_lossy()
                ),
                None => anyhow!("No program found for MIME type '{mime}'"),
            })
    }
}
