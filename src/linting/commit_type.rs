use cruet::Inflector;
use serde::{Deserialize, Serialize};

use crate::Commit;

use super::{Casing, Conf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TypeConf {
    #[serde(rename = "enum")]
    pub types: Vec<String>,
    pub min_length: usize,
    pub max_length: usize,
    pub case: Casing,
}

impl Default for TypeConf {
    fn default() -> Self {
        Self {
            types: vec!["*".to_string()],
            min_length: usize::MIN,
            max_length: u32::MAX as usize,
            case: Casing::default(),
        }
    }
}

pub(super) fn commit_type_invalid(commit: &Commit, config: &Conf) -> bool {
    if config.commit_type.types.contains(&"*".to_string()) {
        false
    } else {
        !config
            .commit_type
            .types
            .contains(&commit.commit_type.to_string())
    }
}

pub(super) fn commit_type_case_invalid(commit: &Commit, config: &Conf) -> bool {
    match config.commit_type.case {
        Casing::Camel => !commit.commit_type.is_camel_case(),
        Casing::Kebab => !commit.commit_type.is_kebab_case(),
        Casing::Pascal => !commit.commit_type.is_pascal_case(),
        Casing::Snake => !commit.commit_type.is_snake_case(),
    }
}

pub(super) fn commit_type_too_short(commit: &Commit, config: &Conf) -> bool {
    commit.commit_type.len() <= config.commit_type.min_length
}

pub(super) fn commit_type_too_long(commit: &Commit, config: &Conf) -> bool {
    commit.commit_type.len() >= config.commit_type.max_length
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
    fn test_invalid() {
        let mut commit = sample_commit();
        commit.commit_type = "invalid";

        let mut config = Conf::default();
        config.commit_type.types = vec![];

        assert!(commit_type_invalid(&commit, &config));
    }

    #[test]
    fn test_valid() {
        let mut commit = sample_commit();
        commit.commit_type = "fix";

        let mut config = Conf::default();
        config.commit_type.types = vec!["fix".to_string()];

        assert!(!commit_type_invalid(&commit, &config));
    }

    #[test]
    fn test_wildcard() {
        let mut commit = sample_commit();
        commit.commit_type = "this-is-very-valid";

        assert!(!commit_type_invalid(&commit, &Conf::default()));
    }

    #[test]
    fn test_too_short() {
        let mut commit = sample_commit();
        commit.commit_type = "fix";

        let mut config = Conf::default();
        config.commit_type.min_length = 4;

        assert!(commit_type_too_short(&commit, &config));
    }

    #[test]
    fn test_too_long() {
        let mut commit = sample_commit();
        commit.commit_type = "fix";

        let mut config = Conf::default();
        config.commit_type.max_length = 2;

        assert!(commit_type_too_long(&commit, &config));
    }

    #[test]
    fn test_long_enough() {
        let mut commit = sample_commit();
        commit.commit_type = "fix";

        let mut config = Conf::default();
        config.commit_type.min_length = 2;

        assert!(!commit_type_too_short(&commit, &config));
    }

    #[test]
    fn test_short_enough() {
        let mut commit = sample_commit();
        commit.commit_type = "fix";

        let mut config = Conf::default();
        config.commit_type.max_length = 4;

        assert!(!commit_type_too_long(&commit, &config));
    }

    #[test]
    fn test_case_invalid() {
        let mut commit = sample_commit();
        commit.commit_type = "snake_case";

        assert!(commit_type_case_invalid(&commit, &Conf::default()));
    }

    #[test]
    fn test_case_valid() {
        let mut commit = sample_commit();
        commit.commit_type = "kebab-case";

        assert!(!commit_type_case_invalid(&commit, &Conf::default()));
    }
}
