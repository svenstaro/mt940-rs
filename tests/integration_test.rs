extern crate mt940;

extern crate serde;
extern crate serde_json;

extern crate rstest;

#[macro_use]
extern crate pretty_assertions;

use mt940::{parse_mt940, Message};
use rstest::rstest_parametrize;
use std::fs;
use std::path::PathBuf;

#[rstest_parametrize(
    statement_path,
    case("danskebank/MT940_DK_Example.sta"),
    case("danskebank/MT940_FI_Example.sta"),
    case("danskebank/MT940_NO_Example.sta"),
    case("danskebank/MT940_SE_Example.sta"),
)]
fn parse_mt940_statement(statement_path: &str) {
    let full_path = PathBuf::from(format!("tests/data/mt940/full/{}", statement_path));
    let input_data = fs::read_to_string(&full_path).unwrap();
    let input_messages = parse_mt940(&input_data).unwrap();

    let expected_data = fs::read_to_string(full_path.with_extension("json")).unwrap();
    let expected_messages: Vec<Message> = serde_json::from_str(&expected_data).unwrap();

    assert_eq!(expected_messages, input_messages);
}
