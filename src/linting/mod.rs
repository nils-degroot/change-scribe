use std::{fmt::Display, path::PathBuf};

use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use miette::{Context, Diagnostic, IntoDiagnostic, Report, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::parsing::parse;
use crate::Commit;
use commit_scope::*;
use commit_type::*;

mod commit_scope;
mod commit_type;

#[derive(Debug, Diagnostic, Error)]
#[error("{kind}")]
struct LintError {
    #[source_code]
    input: String,
    #[label("{}", label.unwrap_or("here"))]
    span: SourceSpan,
    label: Option<&'static str>,
    #[help]
    help: Option<&'static str>,
    kind: LintErrorKind,
}

#[derive(Debug, Diagnostic, Error)]
enum LintErrorKind {
    #[error("Invalid commit type")]
    TypeInvalid,
    #[error("The commit type is too short")]
    TypeTooShort,
    #[error("The commit type is too long")]
    TypeTooLong,
    #[error("Invalid commit type case")]
    TypeCaseInvalid,

    #[error("Scope is required")]
    ScopeRequired,
    #[error("Invalid scope")]
    ScopeInvalid,
    #[error("The scope is too short")]
    ScopeTooShort,
    #[error("The scope is too long")]
    ScopeTooLong,
    #[error("Invalid commit scope case")]
    ScopeCaseInvalid,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct Conf {
    #[serde(rename = "type")]
    commit_type: TypeConf,
    #[serde(rename = "scope")]
    commit_scope: ScopeConf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Casing {
    // camelCase
    Camel,
    // kebab-case
    Kebab,
    // PascalCase
    Pascal,
    // snake_case
    Snake,
}

impl Display for Casing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Casing::Camel => write!(f, "camelCase"),
            Casing::Kebab => write!(f, "kebab-case"),
            Casing::Pascal => write!(f, "PascalCase"),
            Casing::Snake => write!(f, "snake_case"),
        }
    }
}

impl Default for Casing {
    fn default() -> Self {
        Self::Kebab
    }
}

macro_rules! lint_fn {
    ( $( $rule:ident => $error:expr ),* ) => {
        pub(crate) fn lint(message: &'static str, config_path: Option<PathBuf>) -> miette::Result<()> {
            let commit = parse(message)?;

            let config = Figment::new().merge(Serialized::defaults(Conf::default()));

            let config = if let Some(config_path) = config_path {
                config.merge(Toml::file(config_path))
            } else {
                config.merge(Toml::file("change-scribe.toml")).merge(Toml::file(".change-scribe.toml"))
            }.extract::<Conf>().into_diagnostic().context("Failed to load configuration")?;

            let mut errors = Vec::<Report>::new();

            $(
                $rule(&commit, &config).then(|| {
                    errors.push($error(&commit, &config).into());
                });
            )*

            for error in &errors {
                println!("{error:?}");
            }

            if errors.is_empty() {
                Ok(())
            } else {
                miette::bail!("Linting failed")
            }
        }
    };
}

lint_fn! {
    // Type
    commit_type_invalid => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.type_span().into(),
        label: Some("At the commit type"),
        help: Some(format!("Valid types are: {:?}", config.commit_type.types).leak()),
        kind: LintErrorKind::TypeInvalid,
    },
    commit_type_too_short => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.type_span().into(),
        label: Some("At the commit type"),
        help: Some(format!("The commit type must be at least {} characters long", config.commit_type.min_length).leak()),
        kind: LintErrorKind::TypeTooShort,
    },
    commit_type_too_long => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.type_span().into(),
        label: Some("At the commit type"),
        help: Some(format!("The commit type must be at most {} characters long", config.commit_type.max_length).leak()),
        kind: LintErrorKind::TypeTooLong,
    },
    commit_type_case_invalid => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.type_span().into(),
        label: Some("At the commit type"),
        help: Some(format!("The commit type must be in `{}` case", config.commit_type.case).leak()),
        kind: LintErrorKind::TypeCaseInvalid,
    },

    // Scope
    commit_scope_required => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.scope_span().into(),
        label: Some(format!(
            "Insert a scope after the commit type. e.g.: `{}(scope)`",
            commit.commit_type
        ).leak()),
        help: Some(format!("Valid scopes are: {:?}", config.commit_scope.scopes).leak()),
        kind: LintErrorKind::ScopeRequired,
    },
    commit_scope_invalid => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.scope_span().into(),
        label: None,
        help: Some(format!("Valid scopes are: {:?}", config.commit_scope.scopes).leak()),
        kind: LintErrorKind::ScopeInvalid,
    },
    commit_scope_too_short => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.scope_span().into(),
        label: None,
        help: Some(format!("The scope must be at least {} characters long", config.commit_scope.min_length).leak()),
        kind: LintErrorKind::ScopeTooShort,
    },
    commit_scope_too_long => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.scope_span().into(),
        label: None,
        help: Some(format!("The scope must be at most {} characters long", config.commit_scope.max_length).leak()),
        kind: LintErrorKind::ScopeTooLong,
    },
    commit_scope_case_invalid => |commit: &Commit, config: &Conf| LintError {
        input: commit.source.clone(),
        span: commit.scope_span().into(),
        label: None,
        help: Some(format!("The scope must be in `{}` case", config.commit_scope.case).leak()),
        kind: LintErrorKind::ScopeCaseInvalid,
    }
}
