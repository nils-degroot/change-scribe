use serde::{Deserialize, Serialize};

use crate::Commit;

use super::Conf;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScopeConf {
    pub required: bool,
    #[serde(rename = "enum")]
    pub scopes: Vec<String>,
}

impl Default for ScopeConf {
    fn default() -> Self {
        Self {
            required: false,
            scopes: vec!["*".to_string()],
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
}
