use std::collections::HashMap;

use miette::{Diagnostic, SourceSpan};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::anychar,
    combinator::{eof, map, peek},
    multi::many_till,
    sequence::tuple,
    Err, IResult,
};
use thiserror::Error;

use crate::Commit;

#[derive(Debug, Diagnostic, Error)]
#[error("{kind}")]
struct ParseError {
    #[source_code]
    input: String,
    #[label("{}", label.unwrap_or("here"))]
    span: SourceSpan,
    label: Option<&'static str>,
    #[help]
    help: Option<&'static str>,
    kind: ParseErrorKind,
}

#[derive(Debug, Error)]
enum ParseErrorKind {
    #[error("Invalid commit type syntax")]
    ParsingType,
}

pub(crate) fn parse(message: &'static str) -> Result<Commit, miette::Report> {
    let result = parse_internal(message).map_err(|e| {
        let input = match e {
            Err::Error(e) | Err::Failure(e) => e.input,
            Err::Incomplete(_) => message,
        };

        ParseError {
            input: message.to_string(),
            span: (message.len() - input.len(), 0).into(),
            label: None,
            help: None,
            kind: ParseErrorKind::ParsingType,
        }
    })?;

    Ok(result.1)
}

fn parse_internal(message: &'static str) -> IResult<&str, Commit> {
    let (rest, commit_type) = parse_type(message)?;

    let (rest, commit_scope) = match parse_scope(rest) {
        Ok((rest, commit_scope)) => (rest, Some(commit_scope)),
        Err(_) => (rest, None),
    };

    let (rest, breaking_change) = match parse_exclaimation_mark(rest) {
        Ok((rest, _)) => (rest, true),
        Err(_) => (rest, false),
    };

    let (rest, _) = parse_seperator(rest)?;

    let (rest, commit_subject) = parse_subject(rest)?;

    if rest.is_empty() {
        let commit = Commit {
            commit_type,
            scope: commit_scope
                .map(|scope| scope.split(',').collect())
                .unwrap_or_default(),
            breaking_change,
            subject: commit_subject,
            body: None,
            footer: HashMap::new(),
            source: message.to_string(),
        };

        return Ok((rest, commit));
    }

    let mut rest = rest;
    let mut commit_body = String::new();

    while !rest.is_empty() {
        let (new_rest, _) = parse_section_seperator(rest)?;
        rest = new_rest;

        if parse_footer_key(rest).is_ok() {
            break;
        } else {
            let (new_rest, parsed) = parse_body(rest)?;
            rest = new_rest;
            commit_body.push_str(parsed);
        }
    }

    let mut footer = HashMap::<&str, &str>::new();

    while !rest.is_empty() {
        let (new_rest, key) = parse_footer_key(rest)?;
        let (new_rest, value) = parse_footer_value(new_rest)?;
        footer.insert(key, value.leak());

        rest = new_rest;
    }

    let commit = Commit {
        commit_type,
        scope: commit_scope
            .map(|scope| scope.split(',').collect())
            .unwrap_or_default(),
        breaking_change: breaking_change || footer.contains_key("BREAKING CHANGE"),
        subject: commit_subject,
        body: (!commit_body.is_empty()).then_some(commit_body.leak()),
        footer,
        source: message.to_string(),
    };

    Ok((rest, commit))
}

fn parse_type(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphabetic() || c == '-')(input)
}

fn parse_scope(input: &str) -> IResult<&str, &str> {
    map(
        tuple((tag("("), take_until(")"), tag(")"))),
        |(_, text, _)| text,
    )(input)
}

fn parse_exclaimation_mark(input: &str) -> IResult<&str, &str> {
    tag("!")(input)
}

fn parse_seperator(input: &str) -> IResult<&str, &str> {
    tag(": ")(input)
}

fn parse_subject(input: &str) -> IResult<&str, &str> {
    alt((take_until("\n\n"), take_while1(|_| true)))(input)
}

fn parse_section_seperator(input: &str) -> IResult<&str, &str> {
    tag("\n\n")(input)
}

fn parse_body(input: &str) -> IResult<&str, &str> {
    alt((take_until("\n\n"), take_while1(|_| true)))(input)
}

fn parse_footer_key(input: &str) -> IResult<&str, &str> {
    map(
        tuple((
            alt((
                tag("BREAKING CHANGE"),
                take_while1(|c: char| c.is_alphabetic() || c == '-'),
            )),
            alt((tag(": "), tag(" #"))),
        )),
        |(key, _)| key,
    )(input)
}

fn parse_footer_value(input: &str) -> IResult<&str, String> {
    map(
        many_till(anychar, alt((peek(parse_footer_key), eof))),
        |(taken, _)| taken.into_iter().collect::<String>().trim_end().to_string(),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_fixtures {
        ( $( $name:ident => $path:literal ),* ) => {
            $(
                #[test]
                fn $name() {
                    let msg = include_str!($path);
                    parse(msg).unwrap();
                }
            )*
        };
    }

    parse_fixtures! {
        test_description_and_breaking_change_footer => "../fixtures/description-and-breaking-change-footer.txt",
        test_description_and_footer => "../fixtures/description-and-breaking-change-footer.txt",
        test_exclaimation_mark_in_header_with_scope => "../fixtures/exclaimation-mark-in-header-with-scope.txt",
        test_exclaimation_mark_in_header => "../fixtures/exclaimation-mark-in-header.txt",
        test_multiple_paragraphs_in_body_and_multiple_footers => "../fixtures/multi-paragraph-body-and-multiple-footers.txt",
        test_no_body => "../fixtures/no-body.txt",
        test_scope => "../fixtures/scope.txt"
    }

    macro_rules! valid_footer_keys {
        ( $( $name:ident => $value:literal ),* ) => {
            $(
                #[test]
                fn $name() {
                    let (rest, _) = parse_footer_key($value).unwrap();
                    assert_eq!(rest, "");
                }
            )*
        };
    }

    valid_footer_keys! {
        test_breaking_change => "BREAKING CHANGE: ",
        test_breaking_change_with_hash => "BREAKING CHANGE #",
        test_reviewed_by => "Reviewed-by: ",
        test_refs => "Refs: "
    }

    #[test]
    fn terminate_footer_value_on_time() {
        let (rest, key) = parse_footer_key("Reviewed-by: some guy\nRefs: #123").unwrap();
        assert_eq!(key, "Reviewed-by");

        let (rest, value) = parse_footer_value(rest).unwrap();
        assert_eq!(value, "some guy");

        assert_eq!(rest, "Refs: #123");
    }

    #[test]
    fn no_body_turns_into_none() {
        let commit = parse("fix: something\n\nBREAKING CHANGE: yes").unwrap();
        assert_eq!(commit.body, None);
    }

    #[test]
    fn commit_with_body_is_some() {
        let commit = parse("fix: something\n\nChanges were easy\n\nBREAKING CHANGE: yes").unwrap();
        assert_eq!(commit.body, Some("Changes were easy"));
    }

    #[test]
    fn footer_breaking_change_parses_to_breaking_change() {
        let commit = parse("fix: something\n\nBREAKING CHANGE: yes").unwrap();
        assert!(commit.breaking_change);
    }

    #[test]
    fn no_footer_breaking_change_parses_to_false() {
        let commit = parse("fix: something\n\nChanges were easy").unwrap();
        assert!(!commit.breaking_change);
    }
}
