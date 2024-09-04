use cruet::Inflector;
use serde::{Deserialize, Serialize};

use crate::Commit;

use super::{Casing, Conf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ScopeConf {
    pub required: bool,
    #[serde(rename = "enum")]
    pub scopes: Vec<String>,
    pub min_length: usize,
    pub max_length: usize,
    pub case: Casing,
}

impl Default for ScopeConf {
    fn default() -> Self {
        Self {
            required: false,
            scopes: vec!["*".to_string()],
            min_length: usize::MIN,
            max_length: u32::MAX as usize,
            case: Casing::default(),
        }
    }
}

pub(super) fn commit_scope_required(commit: &Commit, config: &Conf) -> bool {
    if config.commit_scope.required {
        commit.scope.is_empty()
    } else {
        false
    }
}

pub(super) fn commit_scope_invalid(commit: &Commit, config: &Conf) -> bool {
    if config.commit_scope.scopes.contains(&"*".to_string()) {
        return false;
    }

    !commit.scope.is_empty()
        && !commit
            .scope
            .iter()
            .any(|scope| config.commit_scope.scopes.contains(&scope.to_string()))
}

pub(super) fn commit_scope_too_short(commit: &Commit, config: &Conf) -> bool {
    commit
        .scope
        .iter()
        .any(|scope| scope.len() <= config.commit_scope.min_length)
}

pub(super) fn commit_scope_too_long(commit: &Commit, config: &Conf) -> bool {
    commit
        .scope
        .iter()
        .any(|scope| scope.len() >= config.commit_scope.max_length)
}

pub(super) fn commit_scope_case_invalid(commit: &Commit, config: &Conf) -> bool {
    match config.commit_scope.case {
        Casing::Camel => commit.scope.iter().any(|scope| !scope.is_camel_case()),
        Casing::Kebab => commit.scope.iter().any(|scope| !scope.is_kebab_case()),
        Casing::Pascal => commit.scope.iter().any(|scope| !scope.is_pascal_case()),
        Casing::Snake => commit.scope.iter().any(|scope| !scope.is_snake_case()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_commit() -> Commit<'static> {
        Commit {
            commit_type: "fix",
            scope: vec![],
            breaking_change: false,
            subject: "subject",
            body: None,
            footer: Default::default(),
            source: "fix subject".to_string(),
        }
    }

    #[test]
    fn test_required() {
        let mut commit = sample_commit();
        commit.scope = vec![];

        let mut config = Conf::default();
        config.commit_scope.required = true;

        assert!(commit_scope_required(&commit, &config));
    }

    #[test]
    fn test_not_required() {
        let mut commit = sample_commit();
        commit.scope = vec![];

        let mut config = Conf::default();
        config.commit_scope.required = false;

        assert!(!commit_scope_required(&commit, &config));
    }

    #[test]
    fn test_invalid() {
        let mut commit = sample_commit();
        commit.scope = vec!["invalid"];

        let mut config = Conf::default();
        config.commit_scope.scopes = vec!["fix".to_string()];

        assert!(commit_scope_invalid(&commit, &config));
    }

    #[test]
    fn test_valid() {
        let mut commit = sample_commit();
        commit.scope = vec!["fix"];

        let mut config = Conf::default();
        config.commit_scope.scopes = vec!["fix".to_string()];

        assert!(!commit_scope_invalid(&commit, &config));
    }

    #[test]
    fn test_wildcard() {
        let mut commit = sample_commit();
        commit.scope = vec!["fix"];

        let mut config = Conf::default();
        config.commit_scope.scopes = vec!["*".to_string()];

        assert!(!commit_scope_invalid(&commit, &config));
    }

    #[test]
    fn test_too_short() {
        let mut commit = sample_commit();
        commit.scope = vec!["fix"];

        let mut config = Conf::default();
        config.commit_scope.min_length = 4;

        assert!(commit_scope_too_short(&commit, &config));
    }

    #[test]
    fn test_too_long() {
        let mut commit = sample_commit();
        commit.scope = vec!["fix"];

        let mut config = Conf::default();
        config.commit_scope.max_length = 2;

        assert!(commit_scope_too_long(&commit, &config));
    }

    #[test]
    fn test_long_enough() {
        let mut commit = sample_commit();
        commit.scope = vec!["fix"];

        let mut config = Conf::default();
        config.commit_scope.max_length = 4;

        assert!(!commit_scope_too_long(&commit, &config));
    }

    #[test]
    fn test_case_invalid() {
        let mut commit = sample_commit();
        commit.scope = vec!["snake_case"];

        assert!(commit_scope_case_invalid(&commit, &Conf::default()));
    }

    #[test]
    fn test_case_valid() {
        let mut commit = sample_commit();
        commit.scope = vec!["kebab-case"];

        assert!(!commit_scope_case_invalid(&commit, &Conf::default()));
    }
}
