use std::{collections::HashMap, path::PathBuf};

use clap::{Parser, Subcommand};
use clap_stdin::MaybeStdin;
use linting::{lint, Conf};
use miette::Context;

mod linting;
mod parsing;

/// A tool that validates that commit messages follow the conventional commit format, and lints
/// them according to a configuration file.
#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    #[clap(short, long)]
    /// Path to the configuration file. Overrides the default configuration.
    config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Lint a commit message.
    Lint {
        /// Message to lint
        message: MaybeStdin<String>,
    },
    /// Commands related to configuration.
    Config {
        #[clap(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Print the default configuration to stdout.
    Dump,
}

fn main() -> Result<(), miette::Report> {
    let args = Args::parse();

    match args.command {
        Command::Lint { message } => {
            lint(Box::new(message.into_inner()).leak(), args.config)?;
        }
        Command::Config { command } => match command {
            ConfigCommand::Dump => {
                let config = default_config()?;
                println!("{}", config);
            }
        },
    }

    Ok(())
}

fn default_config() -> miette::Result<String> {
    let config = Conf::default();

    let config = toml::to_string_pretty(&config)
        .map_err(|e| miette::miette!(e))
        .wrap_err("An error occurred while serializing the default configuration.")?;

    Ok(config)
}

#[derive(Debug)]
#[allow(dead_code)]
struct Commit<'a> {
    commit_type: &'a str,
    scope: Vec<&'a str>,
    breaking_change: bool,
    subject: &'a str,
    body: Option<&'a str>,
    footer: HashMap<&'a str, &'a str>,
    source: String,
}

impl Commit<'_> {
    fn type_span(&self) -> (usize, usize) {
        (0, self.commit_type.len())
    }

    fn scope_span(&self) -> (usize, usize) {
        let start = self.commit_type.len() + 1;
        let end = self.scope.join(":").len();

        (start, end)
    }
}
