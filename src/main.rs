use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use clap_stdin::MaybeStdin;
use linting::lint;

mod linting;
mod parsing;

/// A tool that validates that commit messages follow the conventional commit format, and lints
/// them according to a configuration file.
#[derive(Debug, Parser)]
struct Args {
    /// Message to lint
    message: MaybeStdin<String>,
    #[clap(short, long)]
    /// Path to the configuration file. Overrides the default configuration.
    config: Option<PathBuf>,
}

fn main() -> Result<(), miette::Report> {
    let args = Args::parse();

    lint(Box::new(args.message.into_inner()).leak(), args.config)?;

    Ok(())
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
