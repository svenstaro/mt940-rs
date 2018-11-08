extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate rust_decimal;

mod errors;

use chrono::prelude::*;
use errors::{ParseError, RequiredTagNotFoundError, UnexpectedTagError};
use pest::Parser;
use rust_decimal::Decimal;
use std::fs;

#[derive(Parser)]
#[grammar = "mt940.pest"]
pub struct MT940Parser;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    // Tag :20:
    pub transaction_ref_no: String,

    // Tag :21:
    pub ref_to_related_msg: Option<String>,

    // Tag :25:
    pub account_id: String,

    // Tag :28: or :28C:
    pub statement_no: String,
    pub sequence_no: Option<String>,

    // Tag :60F: or :60M:
    // In case this is :60F: it is the first opening balance.
    // In case of :60M: this is the intermediate opening balance for statements following the
    // first one.
    pub opening_balance: Balance,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    is_intermediate: bool,
    debit_credit_indicator: DebitOrCredit,
    date: NaiveDate,
    iso_currency_code: String,
    amount: Decimal,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebitOrCredit {
    Debit,
    Credit,
}

impl Message {
    /// Construct a new `Message` from a list of fields.
    ///
    /// Must start with field `:20:`. Must not contain more than one `:20:` tag.
    fn from_fields(fields: Vec<Field>) -> Result<Message, ParseError> {
        // Only a few tags may follow after each specific tag.
        let mut current_acceptable_tags = vec!["20"];

        let mut transaction_ref_no = None;
        let mut ref_to_related_msg = None;
        let mut account_id = None;
        let mut statement_no = None;
        let mut sequence_no = None;
        let mut opening_balance = None;

        // For better error reporting.
        let mut last_tag = String::default();

        for field in fields {
            debug!("Now parsing tag: {}", field.tag);

            let current_acceptable_tags_owned = current_acceptable_tags
                .iter()
                .map(|x| x.to_string())
                .collect();
            if !current_acceptable_tags.contains(&&field.tag.as_str()) {
                return Err(UnexpectedTagError::new(
                    field.tag,
                    last_tag,
                    current_acceptable_tags_owned,
                ))?;
            }

            // Set the next acceptable tags.
            match field.tag.as_str() {
                "20" => {
                    let parsed_field = MT940Parser::parse(Rule::tag_20_field, &field.value);
                    transaction_ref_no = Some(parsed_field.unwrap().as_str().to_string());
                    current_acceptable_tags = vec!["21", "25"];
                }
                "21" => {
                    let parsed_field = MT940Parser::parse(Rule::tag_21_field, &field.value);
                    ref_to_related_msg = Some(parsed_field.unwrap().as_str().to_string());
                    current_acceptable_tags = vec!["25"];
                }
                "25" => {
                    let parsed_field = MT940Parser::parse(Rule::tag_25_field, &field.value);
                    account_id = Some(parsed_field.unwrap().as_str().to_string());
                    current_acceptable_tags = vec!["28", "28C"];
                }
                "28C" => {
                    let parsed_field = MT940Parser::parse(Rule::tag_28c_field, &field.value);
                    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
                    for pair in pairs {
                        match pair.as_rule() {
                            Rule::statement_no => statement_no = Some(pair.as_str().to_string()),
                            Rule::sequence_no => sequence_no = Some(pair.as_str().to_string()),
                            _ => (),
                        };
                    }
                    current_acceptable_tags = vec!["60M", "60F"];
                }
                "60M" | "60F" => {
                    let is_intermediate = field.tag.as_str() == "60M";
                    let mut debit_credit_indicator = None;
                    let mut date = None;
                    let mut iso_currency_code = None;
                    let mut amount = None;
                    let parsed_field = MT940Parser::parse(Rule::tag_60_field, &field.value);
                    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
                    for pair in pairs {
                        match pair.as_rule() {
                            Rule::debit_credit_indicator => {
                                if pair.as_str() == "C" {
                                    debit_credit_indicator = Some(DebitOrCredit::Credit);
                                } else if pair.as_str() == "D" {
                                    debit_credit_indicator = Some(DebitOrCredit::Debit);
                                } else {
                                    unreachable!();
                                }
                            }
                            Rule::date => {
                                let mut year = None;
                                let mut month = None;
                                let mut day = None;
                                for p in pair.into_inner() {
                                    match p.as_rule() {
                                        // Here I'm making an assumption that will only work for
                                        // a limited but fairly long time: That all years that we
                                        // see are at least the year 2000 and upwards. The
                                        // problem is sadly that banks didn't make the field be the
                                        // full year number but only a 2-digit number!
                                        // How stupid.
                                        Rule::year => year = Some(format!("20{}", p.as_str())),
                                        Rule::month => month = Some(p.as_str()),
                                        Rule::day => day = Some(p.as_str()),
                                        _ => unreachable!(),
                                    }
                                }
                                date = Some(NaiveDate::from_ymd(
                                    year.unwrap().parse().unwrap(),
                                    month.unwrap().parse().unwrap(),
                                    day.unwrap().parse().unwrap(),
                                ));
                            }
                            Rule::iso_currency_code => {
                                iso_currency_code = Some(pair.as_str().to_string())
                            }
                            Rule::amount => {
                                // Split at decimal separator.
                                let split_decimal_str: Vec<&str> =
                                    pair.as_str().split(",").collect();
                                let (int_part, frac_part) =
                                    (split_decimal_str[0], split_decimal_str[1]);
                                let whole_number: i64 =
                                    format!("{}{}", int_part, frac_part).parse().unwrap();
                                amount = Some(Decimal::new(whole_number, frac_part.len() as u32));
                            }
                            _ => (),
                        };
                    }
                    opening_balance = Some(Balance {
                        is_intermediate,
                        debit_credit_indicator: debit_credit_indicator.unwrap(),
                        date: date.unwrap(),
                        iso_currency_code: iso_currency_code.unwrap(),
                        amount: amount.unwrap(),
                    });
                    current_acceptable_tags = vec!["61", "62M", "62F", "86"];
                }
                "61" => {
                    current_acceptable_tags = vec!["61", "86", "62M", "62F"];
                }
                "86" => {
                    current_acceptable_tags = vec!["61", "62M", "62F", "86"];
                }
                "62M" | "62F" => {
                    current_acceptable_tags = vec!["64", "65", "86"];
                }
                "64" => {
                    current_acceptable_tags = vec!["65", "86"];
                }
                "65" => {
                    current_acceptable_tags = vec!["65", "86"];
                }
                _ => unreachable!(),
            }

            last_tag = field.tag;
        }

        let message = Message {
            transaction_ref_no: transaction_ref_no
                .ok_or(RequiredTagNotFoundError::new("20".to_string()))?,
            ref_to_related_msg: ref_to_related_msg,
            account_id: account_id.ok_or(RequiredTagNotFoundError::new("25".to_string()))?,
            statement_no: statement_no.ok_or(RequiredTagNotFoundError::new("28C".to_string()))?,
            sequence_no: sequence_no,
            opening_balance: opening_balance
                .ok_or(RequiredTagNotFoundError::new("60".to_string()))?,
        };

        Ok(message)
    }
}

/// This is a generic struct that serves as a container for the first pass of the parser.
/// It simply stores every field with absolutely no parsing or validation done on field values.
#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    pub tag: String,
    pub value: String,
}

pub fn parse_fields(statement: &str) -> Result<Vec<Field>, pest::error::Error<Rule>> {
    let parsed_fields = MT940Parser::parse(Rule::fields, statement)?;

    let mut fields = vec![];
    for parsed_field in parsed_fields {
        if let Rule::EOI = parsed_field.as_rule() {
            break;
        }
        let inner = parsed_field.into_inner();
        let tag = inner
            .clone()
            .next()
            .unwrap()
            .into_inner()
            .as_str()
            .to_string();
        let value = inner
            .clone()
            .nth(1)
            .unwrap()
            .as_str()
            .trim()
            .replace("\r\n", "\n")
            .to_string();
        let field = Field { tag, value };
        fields.push(field);
    }

    Ok(fields)
}

pub fn parse_mt940(statement: &str) -> Result<Vec<Message>, ParseError> {
    let fields = parse_fields(statement).map_err(ParseError::PestParseError)?;

    let mut fields_per_message = vec![];

    let mut current_20_tag_index = -1i32;
    for field in fields {
        if field.tag == "20" {
            current_20_tag_index += 1;
            fields_per_message.push(vec![]);
        }
        fields_per_message[current_20_tag_index as usize].push(field);
    }

    let mut messages = vec![];
    for mf in fields_per_message {
        let m = Message::from_fields(mf)?;
        messages.push(m);
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn parse_mt940_tag() {
    //     let expected = "20";
    //     let result = MT940Parser::parse(Rule::tag, ":20:");
    //     assert_eq!(
    //         expected,
    //         result.unwrap().next().unwrap().into_inner().as_str()
    //     );
    // }

    // #[test]
    // fn parse_mt940_record_single_line() {
    //     let expected = Record {
    //         tag: "20".to_string(),
    //         message: "3996-11-11111111".to_string(),
    //     };
    //     let result = parse_mt940(":20:3996-11-11111111\r\n").unwrap();
    //     assert_eq!(expected, result[0]);
    // }
    //
    // #[test]
    // fn parse_mt940_record() {
    //     let expected = Record {
    //         tag: "20".to_string(),
    //         message: "3996-11-11111111\nTES TTEST\nMORETEST".to_string(),
    //     };
    //     let result = parse_mt940(
    //         ":20:3996-11-11111111\r\nTES TTEST\r\nMORETEST\r\n:50:some-other-message\r\n",
    //     )
    //     .unwrap();
    //     assert_eq!(expected, result[0]);
    // }

    #[test]
    fn parse_mt940_statement() {
        let test_data = "asdadad\
                         :20:3996-1234567890\r\n\
                         :25:DABADKKK/1234567890\r\n\
                         :28C:00014/001\r\n\
                         :60F:C091019DKK3859701,48\r\n\
                         :86:For your inform. IBAN no.: DK5030001234567890\r\n\
                         DABADKKK                                                 \r\n\
                         1234567890\r\n\
                         DANSKE BANK                        HOLMENS KANAL 2-12\r\n\
                         :61:0910201020DK5312,50NMSCDBT.teste kunden\r\n\
                         :86:F.M.T.\r\n\
                         V/TESTE KUNDEN\r\n\
                         HOLMENS KANAL 2-12\r\n\
                         1192  KOBENHAVN H\r\n\
                         :61:0910201020DK3009,51NMSCDBT.Pet Van Park\r\n\
                         :86:DBTS 1111272333/Bnf. PET VAN PARK AMSTERDAM/Bnf.acc. NL47ABNAXXXX\r\n\
                         558756/Our fee DKK 40,00/Foreign fee DKK 200,00\r\n\
                         :62F:C091020DKK3851379,47\r\n\
                         :64:C091020DKK3851379,47\r\n\
                         \r\n\
                         ";

        let expected = vec![Message {
            transaction_ref_no: "3996-1234567890".to_string(),
            ref_to_related_msg: None,
            account_id: "DABADKKK/1234567890".to_string(),
        }];

        let result = parse_mt940(test_data).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_mt940_statement_dk_example() {
        let test_data =
            fs::read_to_string("tests/data/self-provided/MT940_DK_Example.sta").unwrap();

        let expected = vec![Message {
            transaction_ref_no: "3996-1234567890".to_string(),
            ref_to_related_msg: None,
            account_id: "DABADKKK/1234567890".to_string(),
        }];

        let result = parse_mt940(&test_data).unwrap();
        assert_eq!(expected, result);
    }
}
