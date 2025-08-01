use pest::error::ErrorVariant;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use std::io;
use std::path::PathBuf;

use mt940::sanitizers::sanitize;
use mt940::{
    parse_mt940, DateParseError, Message, ParseError, RequiredTagNotFoundError, UnexpectedTagError,
};

/// Parse a bunch of MT940 statements that should just work even without sanitation.
#[rstest]
#[case("danskebank/MT940_DK_Example.sta")]
#[case("danskebank/MT940_FI_Example.sta")]
#[case("danskebank/MT940_NO_Example.sta")]
#[case("danskebank/MT940_SE_Example.sta")]
#[case("cmxl/mt940_1.sta")]
fn parse_mt940_statement_success(#[case] statement_path: &str) {
    let full_path = PathBuf::from(format!("tests/data/mt940/full/{statement_path}"));
    let input_data = fs::read_to_string(&full_path).unwrap();
    let parsed_messaged = parse_mt940(&input_data).unwrap();

    let expected_data = fs::read_to_string(full_path.with_extension("json")).unwrap();
    let expected_messages: Vec<Message> = serde_json::from_str(&expected_data).unwrap();

    assert_eq!(expected_messages, parsed_messaged);
}

/// Parse a bunch of MT940 statements that only work after sanitation.
#[rstest]
#[case("betterplace/sepa_mt9401.sta")]
#[case("betterplace/sepa_snippet.sta")]
#[case("betterplace/with_binary_character.sta")]
#[case("cmxl/mt940_2.sta")]
#[case("jejik/abnamro.sta")]
#[case("jejik/ing.sta")]
#[case("jejik/knab.sta")]
#[case("jejik/postfinance.sta")]
#[case("jejik/rabobank-iban.sta")]
#[case("jejik/sns.sta")]
#[case("mBank/mt940.sta")]
#[case("mBank/with_newline_in_tnr.sta")]
#[case("sparkasse/buxtehude.sta")]
#[case("abnamro/mt940.sta")]
#[case("bugs/issue-51.sta")]
fn parse_mt940_statement_success_with_sanitation(#[case] statement_path: &str) {
    let full_path = PathBuf::from(format!("tests/data/mt940/full/{statement_path}"));
    let input_data = fs::read_to_string(&full_path).unwrap();
    let parsed_messages = parse_mt940(&sanitize(&input_data)).unwrap();

    let expected_data = fs::read_to_string(full_path.with_extension("json")).unwrap();
    let expected_messages: Vec<Message> = serde_json::from_str(&expected_data).unwrap();

    assert_eq!(expected_messages, parsed_messages);
}

/// Parse a bunch of invalid statements that should fail even with sanitation.
#[rstest(
    statement_path,
    case("betterplace/sepa_snippet_broken.sta"),
    case("jejik/knab_broken.sta")
)]
fn parse_mt940_statement_fail(statement_path: &str) {
    let full_path = PathBuf::from(format!("tests/data/mt940/full/{statement_path}"));
    let input_data = fs::read_to_string(&full_path).unwrap();
    let parsed_messages = parse_mt940(&sanitize(&input_data));

    assert!(parsed_messages.is_err());
}

#[test]
fn fail_no_tag_20() {
    let input_data = "http://example.com";
    let parsed = parse_mt940(input_data);
    let expected = RequiredTagNotFoundError::new("20");
    if let Err(ParseError::RequiredTagNotFoundError(e)) = parsed {
        assert_eq!(e, expected);
        return;
    }
    assert!(false);
}

#[test]
fn fail_februrary_30() {
    let input_data = fs::read_to_string("tests/data/mt940/special-cases/february_30.sta").unwrap();
    let parsed = parse_mt940(&input_data);
    let expected = ParseError::DateParseError(Box::new(DateParseError::OutOfRange {
        year: "2016".to_string(),
        month: "02".to_string(),
        day: "30".to_string(),
    }));
    assert_eq!(parsed, Err(expected));
}

#[test]
fn fail_incomplete_tag_61() {
    let input_data =
        fs::read_to_string("tests/data/mt940/special-cases/incomplete_tag_61.sta").unwrap();
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
            let e = format!("{e}");
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
                UnexpectedTagError::new("28C", "20", vec!["21".to_string(), "25".to_string()])
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
