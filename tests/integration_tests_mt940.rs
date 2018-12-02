extern crate mt940;

extern crate pest;
extern crate serde;
extern crate serde_json;

extern crate rstest;

#[macro_use]
extern crate pretty_assertions;
use pest::error::ErrorVariant;
use rstest::rstest_parametrize;
use std::fs;
use std::io;
use std::path::PathBuf;

use mt940::{
    parse_mt940, DateParseError, Message, ParseError, RequiredTagNotFoundError, UnexpectedTagError,
};

#[rstest_parametrize(
    statement_path,
    case("danskebank/MT940_DK_Example.sta"),
    case("danskebank/MT940_FI_Example.sta"),
    case("danskebank/MT940_NO_Example.sta"),
    case("danskebank/MT940_SE_Example.sta")
)]
fn parse_mt940_statement_success(statement_path: &str) {
    let full_path = PathBuf::from(format!("tests/data/mt940/full/{}", statement_path));
    let input_data = fs::read_to_string(&full_path).unwrap();
    let input_messages = parse_mt940(&input_data).unwrap();

    let expected_data = fs::read_to_string(full_path.with_extension("json")).unwrap();
    let expected_messages: Vec<Message> = serde_json::from_str(&expected_data).unwrap();

    assert_eq!(expected_messages, input_messages);
}

#[test]
fn fail_februrary_30() {
    let input_data = fs::read_to_string("tests/data/mt940/special-cases/february_30.sta").unwrap();
    let parsed = parse_mt940(&input_data);
    let expected = ParseError::DateParseError(DateParseError::OutOfRange {
        year: "2016".to_string(),
        month: "02".to_string(),
        day: "30".to_string(),
    });
    assert_eq!(parsed, Err(expected));
}

#[test]
fn fail_incomplete_tag61() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/incomplete_tag61.sta").unwrap();
    if let Err(e) = parse_mt940(&input_data) {
        if let ParseError::PestParseError(e) = e {
            if let ErrorVariant::ParsingError {
                positives: _,
                negatives: _,
            } = e.variant
            {
                assert!(true);
                return;
            }
        }
    }
    assert!(false);
}

#[test]
fn fail_invalid_statement() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/invalid_statement.sta").unwrap();
    if let Err(e) = parse_mt940(&input_data) {
        if let ParseError::RequiredTagNotFoundError(e) = e {
            assert_eq!(e, RequiredTagNotFoundError::new("20"));
            return;
        }
    }
    assert!(false);
}

#[test]
fn fail_overly_long_details() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/overly_long_details.sta").unwrap();
    if let Err(e) = parse_mt940(&input_data) {
        if let ParseError::PestParseError(e) = e {
            let e = format!("{}", e);
            assert!(e.contains("?33g Erhebung?34992?60000000012345 BIC: BYLADEMM "));
            return;
        }
    }
    assert!(false);
}

#[test]
fn fail_invalid_utf8() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/invalid_utf8.sta").unwrap_err();
    assert_eq!(input_data.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn fail_unexpected_tag() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/unexpected_tag.sta").unwrap();
    if let Err(e) = parse_mt940(&input_data) {
        if let ParseError::UnexpectedTagError(e) = e {
            assert_eq!(
                e,
                UnexpectedTagError::new("28C", "20", &vec!["21".to_string(), "25".to_string()])
            );
            return;
        }
    }
    assert!(false);
}

#[test]
fn fail_unknown_tag() {
    let input_data = fs::read_to_string("tests/data/mt940/special-cases/unknown_tag.sta").unwrap();
    if let Err(e) = parse_mt940(&input_data) {
        assert_eq!(e, ParseError::UnknownTagError("12".to_string()));
        return;
    }
    assert!(false);
}
