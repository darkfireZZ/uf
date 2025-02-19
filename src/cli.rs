use {
    crate::Config,
    anyhow::{anyhow, Context},
    std::{
        env, io,
        os::unix::process::CommandExt,
        process::{self, Command},
    },
};

macro_rules! usage {
    () => {
        concat!("Usage: ", env!("CARGO_PKG_NAME"), " <FILE>\n")
    };
}

macro_rules! error_help_body {
    () => {
        concat!(
            "Try '",
            env!("CARGO_PKG_NAME"),
            " --help' for more information.\n"
        )
    };
}

macro_rules! help_body {
    () => {
        r#"
Open FILE with the appropriate program

Options:
  -h, --help     Print this help message and exit
  -v, --version  Print the version number and exit
"#
    };
}

const HELP: &str = concat!(usage!(), help_body!());
const ERROR_HELP: &str = concat!(usage!(), error_help_body!());

#[derive(Debug)]
pub struct Cli {
    file: String,
}

impl Cli {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args_os()
            .map(|arg| {
                arg.into_string().unwrap_or_else(|arg| {
                    eprintln!("Error: {:?}", arg);
                    process::exit(1);
                })
            })
            .collect();

        if args.len() < 2 {
            Self::print_error_help(io::stderr());
            process::exit(1);
        }

        if args[1] == "-h" || args[1] == "--help" {
            Self::print_help(io::stdout());
            process::exit(0);
        } else if args[1] == "-v" || args[1] == "--version" {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }

        if args.len() != 2 {
            Self::print_error_help(io::stderr());
            process::exit(1);
        }

        Self {
            file: args[1].clone(),
        }
    }

    fn print_help<W: io::Write>(mut out: W) {
        let _ = out.write_all(HELP.as_bytes());
    }

    fn print_error_help<W: io::Write>(mut out: W) {
        let _ = out.write_all(ERROR_HELP.as_bytes());
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let config = Config::load()?;
        let program = config
            .get_program(&self.file)?
            .ok_or_else(|| anyhow!("No program set for the given file type"))?;
        Err(Command::new(program).arg(&self.file).exec()).context("Failed to open the file")
    }
}
