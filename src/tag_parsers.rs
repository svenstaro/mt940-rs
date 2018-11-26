use std::str::FromStr;

use chrono::prelude::*;
use pest::Parser;

use errors::RequiredTagNotFoundError;
use utils::{date_from_mt940_date, decimal_from_mt940_amount};
use MT940Parser;
use Rule;
use {
    AvailableBalance, Balance, DebitOrCredit, ExtDebitOrCredit, Field, ParseError, StatementLine,
    TransactionTypeIdentificationCode,
};

pub fn parse_20_tag(field: &Field) -> Result<String, ParseError> {
    if field.tag != "20" {
        Err(RequiredTagNotFoundError::new("20"))?;
    }
    let parsed_field = MT940Parser::parse(Rule::tag_20_field, &field.value);
    let transaction_ref_no = parsed_field?.as_str().to_string();
    Ok(transaction_ref_no)
}

pub fn parse_21_tag(field: &Field) -> Result<String, ParseError> {
    if field.tag != "21" {
        Err(RequiredTagNotFoundError::new("21"))?;
    }
    let parsed_field = MT940Parser::parse(Rule::tag_21_field, &field.value);
    let ref_to_related_msg = parsed_field.unwrap().as_str().to_string();
    Ok(ref_to_related_msg)
}

pub fn parse_25_tag(field: &Field) -> Result<String, ParseError> {
    if field.tag != "25" {
        Err(RequiredTagNotFoundError::new("21"))?;
    }
    let parsed_field = MT940Parser::parse(Rule::tag_25_field, &field.value);
    let account_id = parsed_field?.as_str().to_string();
    Ok(account_id)
}

pub fn parse_28c_tag(field: &Field) -> Result<(String, Option<String>), ParseError> {
    if field.tag != "28C" {
        Err(RequiredTagNotFoundError::new("28C"))?;
    }
    let mut statement_no = None;
    let mut sequence_no = None;
    let parsed_field = MT940Parser::parse(Rule::tag_28c_field, &field.value);
    let pairs = parsed_field?.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::statement_no => statement_no = Some(pair.as_str().to_string()),
            Rule::sequence_no => sequence_no = Some(pair.as_str().to_string()),
            _ => (),
        };
    }
    Ok((statement_no.unwrap(), sequence_no))
}

pub fn parse_60_tag(field: &Field) -> Result<Balance, ParseError> {
    if field.tag != "60M" && field.tag != "60F" {
        Err(RequiredTagNotFoundError::new("60"))?;
    }
    let is_intermediate = field.tag.as_str() == "60M";
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_60_field, &field.value);
    let pairs = parsed_field?.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::debit_credit_indicator => {
                debit_credit_indicator = Some(DebitOrCredit::from_str(pair.as_str()).unwrap());
            }
            Rule::date => date = Some(date_from_mt940_date(pair.as_str()).unwrap()),
            Rule::iso_currency_code => iso_currency_code = Some(pair.as_str().to_string()),
            Rule::amount => {
                amount = Some(decimal_from_mt940_amount(pair.as_str()).unwrap());
            }
            _ => (),
        };
    }
    let opening_balance = Balance {
        is_intermediate,
        debit_credit_indicator: debit_credit_indicator.unwrap(),
        date: date.unwrap(),
        iso_currency_code: iso_currency_code.unwrap(),
        amount: amount.unwrap(),
    };
    Ok(opening_balance)
}

pub fn parse_61_tag(field: &Field) -> Result<StatementLine, ParseError> {
    if field.tag != "61" {
        Err(RequiredTagNotFoundError::new("61"))?;
    }
    let mut date = None;
    let mut short_date = None;
    let mut ext_debit_credit_indicator = None;
    let mut funds_code = None;
    let mut amount = None;
    let mut transaction_type_ident_code = None;
    let mut customer_ref = None;
    let mut bank_ref = None;
    let mut supplementary_details = None;
    let parsed_field = MT940Parser::parse(Rule::tag_61_field, &field.value);
    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::date => date = Some(date_from_mt940_date(pair.as_str()).unwrap()),
            Rule::short_date => {
                let mut month = None;
                let mut day = None;
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::month => month = Some(p.as_str()),
                        Rule::day => day = Some(p.as_str()),
                        _ => unreachable!(),
                    }
                }
                // Since we only get month and day from the short date, we'll have
                // to make an assumption about the year.
                // We'll assume that this is in the same year as the statement
                // line's year. This might result in some cases where the
                // statement's year is 2018-12-31 and the entry is given as 0101
                // which would then result in this the entry date ending up as
                // 2018-01-01 even though it should be 2019-01-01. I'll not be too
                // smart about this for now but I'll keep an eye on this.
                short_date = Some(NaiveDate::from_ymd(
                    date.unwrap().year(),
                    month.unwrap().parse().unwrap(),
                    day.unwrap().parse().unwrap(),
                ));
            }
            Rule::ext_debit_credit_indicator => {
                ext_debit_credit_indicator =
                    Some(ExtDebitOrCredit::from_str(pair.as_str()).unwrap());
            }
            Rule::funds_code => {
                funds_code = Some(pair.as_str().to_string());
            }
            Rule::amount => {
                amount = Some(decimal_from_mt940_amount(pair.as_str()).unwrap());
            }
            Rule::transaction_type_ident_code => {
                // The actual transaction type ident code begins after the first
                // character. The first character is either "N" or "F".
                let actual_type_ident_code_str = &pair.as_str()[1..];
                match TransactionTypeIdentificationCode::from_str(actual_type_ident_code_str) {
                    Ok(t) => transaction_type_ident_code = Some(t),
                    Err(strum::ParseError::VariantNotFound) => {
                        return Err(ParseError::InvalidTransactionIdentCode(
                            pair.as_str().to_string(),
                        ))
                    }
                };
            }
            Rule::customer_ref => {
                customer_ref = Some(pair.as_str().to_string());
            }
            Rule::bank_ref => {
                bank_ref = Some(pair.as_str().to_string());
            }
            Rule::supplementary_details => {
                supplementary_details = Some(pair.as_str().to_string());
            }
            _ => (),
        }
    }
    let statement_line = StatementLine {
        value_date: date.unwrap(),
        entry_date: short_date,
        ext_debit_credit_indicator: ext_debit_credit_indicator.unwrap(),
        funds_code: funds_code,
        amount: amount.unwrap(),
        transaction_type_ident_code: transaction_type_ident_code.unwrap(),
        customer_ref: customer_ref.unwrap(),
        bank_ref: bank_ref,
        supplementary_details: supplementary_details,
        information_to_account_owner: None,
    };
    Ok(statement_line)
}

pub fn parse_86_tag(field: &Field) -> Result<String, ParseError> {
    if field.tag != "86" {
        Err(RequiredTagNotFoundError::new("86"))?;
    }
    let parsed_field = MT940Parser::parse(Rule::tag_86_field, &field.value);
    let information_to_account_owner = parsed_field?.as_str().to_string();
    Ok(information_to_account_owner)
}

pub fn parse_62_tag(field: &Field) -> Result<Balance, ParseError> {
    if field.tag != "62M" && field.tag != "62F" {
        Err(RequiredTagNotFoundError::new("62"))?;
    }
    let is_intermediate = field.tag.as_str() == "62M";
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_62_field, &field.value);
    let pairs = parsed_field?.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::debit_credit_indicator => {
                debit_credit_indicator = Some(DebitOrCredit::from_str(pair.as_str()).unwrap());
            }
            Rule::date => date = Some(date_from_mt940_date(pair.as_str()).unwrap()),
            Rule::iso_currency_code => iso_currency_code = Some(pair.as_str().to_string()),
            Rule::amount => {
                amount = Some(decimal_from_mt940_amount(pair.as_str()).unwrap());
            }
            _ => (),
        };
    }
    let closing_balance = Balance {
        is_intermediate,
        debit_credit_indicator: debit_credit_indicator.unwrap(),
        date: date.unwrap(),
        iso_currency_code: iso_currency_code.unwrap(),
        amount: amount.unwrap(),
    };
    Ok(closing_balance)
}

pub fn parse_64_tag(field: &Field) -> Result<AvailableBalance, ParseError> {
    if field.tag != "64" {
        Err(RequiredTagNotFoundError::new("64"))?;
    }
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_64_field, &field.value);
    let pairs = parsed_field?.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::debit_credit_indicator => {
                debit_credit_indicator = Some(DebitOrCredit::from_str(pair.as_str()).unwrap());
            }
            Rule::date => date = Some(date_from_mt940_date(pair.as_str()).unwrap()),
            Rule::iso_currency_code => iso_currency_code = Some(pair.as_str().to_string()),
            Rule::amount => {
                amount = Some(decimal_from_mt940_amount(pair.as_str()).unwrap());
            }
            _ => (),
        };
    }
    let closing_available_balance = AvailableBalance {
        debit_credit_indicator: debit_credit_indicator.unwrap(),
        date: date.unwrap(),
        iso_currency_code: iso_currency_code.unwrap(),
        amount: amount.unwrap(),
    };
    Ok(closing_available_balance)
}

pub fn parse_65_tag(field: &Field) -> Result<AvailableBalance, ParseError> {
    if field.tag != "65" {
        Err(RequiredTagNotFoundError::new("65"))?;
    }
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_65_field, &field.value);
    let pairs = parsed_field?.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::debit_credit_indicator => {
                debit_credit_indicator = Some(DebitOrCredit::from_str(pair.as_str()).unwrap());
            }
            Rule::date => date = Some(date_from_mt940_date(pair.as_str()).unwrap()),
            Rule::iso_currency_code => iso_currency_code = Some(pair.as_str().to_string()),
            Rule::amount => {
                amount = Some(decimal_from_mt940_amount(pair.as_str()).unwrap());
            }
            _ => (),
        };
    }
    let forward_available_balance = AvailableBalance {
        debit_credit_indicator: debit_credit_indicator.unwrap(),
        date: date.unwrap(),
        iso_currency_code: iso_currency_code.unwrap(),
        amount: amount.unwrap(),
    };
    Ok(forward_available_balance)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use regex::Regex;
    use rstest::rstest_parametrize;
    use rust_decimal::Decimal;
    use strum::IntoEnumIterator;

    use super::*;

    proptest! {
        #[test]
        fn tag_20_input(input in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{1, 16}") {
            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&input), "Can't have a value that looks like a tag");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&input), "Can't have a value that has whitespace in front or end");

            let field = Field::from_str(&format!(":20:{}", input)).unwrap();
            let parsed = parse_20_tag(&field).unwrap();
            prop_assert_eq!(&parsed, &input);
        }
    }

    proptest! {
        #[test]
        fn tag_21_input(input in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{1, 16}") {
            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&input), "Can't have a value that looks like a tag");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&input), "Can't have a value that has whitespace in front or end");

            let field = Field::from_str(&format!(":21:{}", input)).unwrap();
            let parsed = parse_21_tag(&field).unwrap();
            prop_assert_eq!(&parsed, &input);
        }
    }

    proptest! {
        #[test]
        fn tag_25_input(input in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{1, 35}") {
            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&input), "Can't have a value that looks like a tag");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&input), "Can't have a value that has whitespace in front or end");

            let field = Field::from_str(&format!(":25:{}", input)).unwrap();
            let parsed = parse_25_tag(&field).unwrap();
            prop_assert_eq!(&parsed, &input);
        }
    }

    proptest! {
        #[test]
        fn tag_28c_input(statement_no in r"[[:digit:]]{1, 5}",
                         sequence_no in r"[[:digit:]]{0, 5}") {
            let input = format!(
                "{statement_no}{separator}{sequence_no}",
                statement_no=statement_no,
                separator=if sequence_no.is_empty() { "" } else { "/" },
                sequence_no=sequence_no);

            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&input), "Can't have a value that looks like a tag");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&input), "Can't have a value that has whitespace in front or end");

            let field = Field::from_str(&format!(":28C:{}", input)).unwrap();
            let parsed = parse_28c_tag(&field).unwrap();
            let expected = (
                statement_no,
                if sequence_no.is_empty() {
                    None
                } else {
                    Some(sequence_no)
                }
            );
            prop_assert_eq!(parsed, expected);
        }
    }

    #[rstest_parametrize(
        input,
        expected_decimal,
        case(":60F:C100318EUR380115,12", "380115.12"),
        case(":60F:C100318EUR380115,1", "380115.10"),
        case(":60F:C100318EUR380115,", "380115.00"),
        case(":60F:C100318EUR0,12", "0.12"),
        case(":60F:C100318EUR00,12", "0.12"),
        case(":60F:C100318EUR001,12", "1.12")
    )]
    fn tag_60_input_specific(input: &str, expected_decimal: &str) {
        let expected = Balance {
            is_intermediate: false,
            debit_credit_indicator: DebitOrCredit::Credit,
            date: NaiveDate::from_ymd(2010, 3, 18),
            iso_currency_code: "EUR".into(),
            amount: Decimal::from_str(expected_decimal).unwrap(),
        };
        let field = Field::from_str(input).unwrap();
        let parsed = parse_60_tag(&field).unwrap();
        assert_eq!(parsed, expected);
    }

    proptest! {
        #[test]
        fn tag_60_input(intermediate in r"[MF]",
                        debit_credit_indicator in r"[DC]",
                        date in r"[[:digit:]]{2}[01][0-9][0-3][[:digit:]]",
                        iso_currency_code in r"[[:alpha:]]{3}",
                        amount_before_decimal in r"[[:digit:]]{1, 12}",
                        amount_after_decimal in r"[[:digit:]]{0, 2}") {
            prop_assume!(NaiveDate::parse_from_str(&date, "%y%m%d").is_ok(), "We need a valid date");

            let amount = format!("{},{}", amount_before_decimal, amount_after_decimal);
            let input = format!(
                "{debit_credit_indicator}{date}{iso_currency_code}{amount}",
                debit_credit_indicator=debit_credit_indicator,
                date=date,
                iso_currency_code=iso_currency_code,
                amount=amount);

            let field = Field::from_str(&format!(":60{}:{}", intermediate, input)).unwrap();
            let parsed = parse_60_tag(&field).unwrap();
            let expected = Balance {
                is_intermediate: if intermediate == "M" { true } else { false },
                debit_credit_indicator: DebitOrCredit::from_str(&debit_credit_indicator).unwrap(),
                date: date_from_mt940_date(&date).unwrap(),
                iso_currency_code: iso_currency_code,
                amount: decimal_from_mt940_amount(&amount).unwrap(),
            };
            prop_assert_eq!(parsed, expected);
        }
    }

    proptest! {
        #[test]
        fn tag_62_input(intermediate in r"[MF]",
                        debit_credit_indicator in r"[DC]",
                        date in r"[[:digit:]]{2}[01][0-9][0-3][[:digit:]]",
                        iso_currency_code in r"[[:alpha:]]{3}",
                        amount_before_decimal in r"[[:digit:]]{1, 12}",
                        amount_after_decimal in r"[[:digit:]]{0, 2}") {
            prop_assume!(NaiveDate::parse_from_str(&date, "%y%m%d").is_ok(), "We need a valid date");

            let amount = format!("{},{}", amount_before_decimal, amount_after_decimal);
            let input = format!(
                "{debit_credit_indicator}{date}{iso_currency_code}{amount}",
                debit_credit_indicator=debit_credit_indicator,
                date=date,
                iso_currency_code=iso_currency_code,
                amount=amount);

            let field = Field::from_str(&format!(":62{}:{}", intermediate, input)).unwrap();
            let parsed = parse_62_tag(&field).unwrap();
            let expected = Balance {
                is_intermediate: if intermediate == "M" { true } else { false },
                debit_credit_indicator: DebitOrCredit::from_str(&debit_credit_indicator).unwrap(),
                date: date_from_mt940_date(&date).unwrap(),
                iso_currency_code: iso_currency_code,
                amount: decimal_from_mt940_amount(&amount).unwrap(),
            };
            prop_assert_eq!(parsed, expected);
        }
    }

    proptest! {
        #[test]
        fn tag_64_input(debit_credit_indicator in r"[DC]",
                        date in r"[[:digit:]]{2}[01][0-9][0-3][[:digit:]]",
                        iso_currency_code in r"[[:alpha:]]{3}",
                        amount_before_decimal in r"[[:digit:]]{1, 12}",
                        amount_after_decimal in r"[[:digit:]]{0, 2}") {
            prop_assume!(NaiveDate::parse_from_str(&date, "%y%m%d").is_ok(), "We need a valid date");

            let amount = format!("{},{}", amount_before_decimal, amount_after_decimal);
            let input = format!(
                "{debit_credit_indicator}{date}{iso_currency_code}{amount}",
                debit_credit_indicator=debit_credit_indicator,
                date=date,
                iso_currency_code=iso_currency_code,
                amount=amount);

            let field = Field::from_str(&format!(":64:{}", input)).unwrap();
            let parsed = parse_64_tag(&field).unwrap();
            let expected = AvailableBalance {
                debit_credit_indicator: DebitOrCredit::from_str(&debit_credit_indicator).unwrap(),
                date: date_from_mt940_date(&date).unwrap(),
                iso_currency_code: iso_currency_code,
                amount: decimal_from_mt940_amount(&amount).unwrap(),
            };
            prop_assert_eq!(parsed, expected);
        }
    }

    proptest! {
        #[test]
        fn tag_65_input(debit_credit_indicator in r"[DC]",
                        date in r"[[:digit:]]{2}[01][0-9][0-3][[:digit:]]",
                        iso_currency_code in r"[[:alpha:]]{3}",
                        amount_before_decimal in r"[[:digit:]]{1, 12}",
                        amount_after_decimal in r"[[:digit:]]{0, 2}") {
            prop_assume!(NaiveDate::parse_from_str(&date, "%y%m%d").is_ok(), "We need a valid date");

            let amount = format!("{},{}", amount_before_decimal, amount_after_decimal);
            let input = format!(
                "{debit_credit_indicator}{date}{iso_currency_code}{amount}",
                debit_credit_indicator=debit_credit_indicator,
                date=date,
                iso_currency_code=iso_currency_code,
                amount=amount);

            let field = Field::from_str(&format!(":65:{}", input)).unwrap();
            let parsed = parse_65_tag(&field).unwrap();
            let expected = AvailableBalance {
                debit_credit_indicator: DebitOrCredit::from_str(&debit_credit_indicator).unwrap(),
                date: date_from_mt940_date(&date).unwrap(),
                iso_currency_code: iso_currency_code,
                amount: decimal_from_mt940_amount(&amount).unwrap(),
            };
            prop_assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn tag_61_empty_entry_date() {
        let field = Field::from_str(":61:110701CN50,00NDISNONREF").unwrap();
        let parsed = parse_61_tag(&field).unwrap();
        assert_eq!(parsed.entry_date, None);
    }

    proptest! {
        #[test]
        fn tag_61_input(date in (r"[[:digit:]]{2}[01][0-9][0-3][[:digit:]]").prop_filter("We need a valid date", |d| NaiveDate::parse_from_str(&d, "%y%m%d").is_ok()),
                        has_short_date in proptest::bool::weighted(0.5),
                        ext_debit_credit_indicator in r"R?[DC]",
                        funds_code in r"[[:alpha:]]?",
                        amount_before_decimal in r"[[:digit:]]{1, 12}",
                        amount_after_decimal in r"[[:digit:]]{0, 2}",
                        transaction_type_ident_code_nf in r"[NF]",
                        transaction_type_ident_code_enum in proptest::sample::select(
                            TransactionTypeIdentificationCode::iter()
                            .map(|x| format!("{:?}", x))
                            .collect::<Vec<String>>()
                        ),
                        customer_ref in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{1, 16}",
                        bank_ref in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{0, 16}",
                        supplementary_details in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{0, 34}") {
            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&customer_ref), "Can't have a value that looks like a tag");
            prop_assume!(!re_tag_like.is_match(&bank_ref), "Can't have a value that looks like a tag");
            prop_assume!(!re_tag_like.is_match(&supplementary_details), "Can't have a value that looks like a tag");

            let re_bank_ref_separator = Regex::new(r"(//)").unwrap();
            prop_assume!(!re_bank_ref_separator.is_match(&customer_ref), "Can't have a value that looks like a separator");

            let re_slash_at_end = Regex::new(r"/$").unwrap();
            prop_assume!(!re_slash_at_end.is_match(&customer_ref), "Can't have a customer ref that ends in a slash");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&customer_ref), "Can't have a value that has whitespace in front or end");
            prop_assume!(re_no_ws_in_front_or_end.is_match(&bank_ref), "Can't have a value that has whitespace in front or end");
            prop_assume!(re_no_ws_in_front_or_end.is_match(&supplementary_details), "Can't have a value that has whitespace in front or end");

            let short_date = if has_short_date { &date[2..6] } else { "" };
            let amount = format!("{},{}", amount_before_decimal, amount_after_decimal);
            let transaction_type_ident_code = format!(
                "{}{}",
                transaction_type_ident_code_nf,
                transaction_type_ident_code_enum);
            let customer_bank_ref = format!(
                "{customer_ref}{separator}{bank_ref}",
                customer_ref=customer_ref,
                separator=if bank_ref.is_empty() { "" } else { "//" },
                bank_ref=bank_ref);

            let input = format!(
                "{date}{short_date}{ext_debit_credit_indicator}{funds_code}\
                 {amount}{transaction_type_ident_code}{customer_bank_ref}\
                 \n{supplementary_details}",
                ext_debit_credit_indicator=ext_debit_credit_indicator,
                date=date,
                short_date=short_date,
                funds_code=funds_code,
                amount=amount,
                transaction_type_ident_code=transaction_type_ident_code,
                customer_bank_ref=customer_bank_ref,
                supplementary_details=supplementary_details);
            let field = Field::from_str(&format!(":61:{}", input)).unwrap();
            let parsed = parse_61_tag(&field).unwrap();
            let expected = StatementLine {
                value_date: date_from_mt940_date(&date).unwrap(),
                entry_date: if has_short_date { Some(date_from_mt940_date(&date).unwrap()) } else { None },
                ext_debit_credit_indicator: ExtDebitOrCredit::from_str(&ext_debit_credit_indicator).unwrap(),
                funds_code: if funds_code.is_empty() { None } else { Some(funds_code) },
                amount: decimal_from_mt940_amount(&amount).unwrap(),
                transaction_type_ident_code: TransactionTypeIdentificationCode::from_str(&transaction_type_ident_code_enum).unwrap(),
                customer_ref,
                bank_ref: if bank_ref.is_empty() { None } else { Some(bank_ref) },
                supplementary_details: if supplementary_details.is_empty() { None } else { Some(supplementary_details) },
                information_to_account_owner: None,
            };
            prop_assert_eq!(parsed, expected);
        }
    }

    proptest! {
        #[test]
        fn tag_86_input(information_to_account_owner_count in 1..6,
                        information_to_account_owner_text in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]{1, 65}") {
            let information_to_account_owner = (0..information_to_account_owner_count)
                .map(|_| information_to_account_owner_text.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&information_to_account_owner), "Can't have a value that looks like a tag");

            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&information_to_account_owner),
                "Can't have a value that has whitespace in front or end");

            let field = Field::from_str(&format!(":86:{}", information_to_account_owner)).unwrap();
            let parsed = parse_86_tag(&field).unwrap();
            prop_assert_eq!(parsed, information_to_account_owner);
        }
    }
}
