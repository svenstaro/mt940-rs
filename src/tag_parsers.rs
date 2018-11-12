use std::str::FromStr;

use chrono::prelude::*;
use pest::Parser;

use utils::{date_from_mt940_date, decimal_from_mt940_amount};
use MT940Parser;
use Rule;
use {
    AvailableBalance, Balance, DebitOrCredit, ExtDebitOrCredit, Field, ParseError, StatementLine,
    TransactionTypeIdentificationCode,
};

pub fn parse_20_tag(field: &Field) -> String {
    let parsed_field = MT940Parser::parse(Rule::tag_20_field, &field.value);
    let transaction_ref_no = parsed_field.unwrap().as_str().to_string();
    transaction_ref_no
}

pub fn parse_21_tag(field: &Field) -> String {
    let parsed_field = MT940Parser::parse(Rule::tag_21_field, &field.value);
    let ref_to_related_msg = parsed_field.unwrap().as_str().to_string();
    ref_to_related_msg
}

pub fn parse_25_tag(field: &Field) -> String {
    let parsed_field = MT940Parser::parse(Rule::tag_25_field, &field.value);
    let account_id = parsed_field.unwrap().as_str().to_string();
    account_id
}

pub fn parse_28c_tag(field: &Field) -> (Option<String>, Option<String>) {
    let mut statement_no = None;
    let mut sequence_no = None;
    let parsed_field = MT940Parser::parse(Rule::tag_28c_field, &field.value);
    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::statement_no => statement_no = Some(pair.as_str().to_string()),
            Rule::sequence_no => sequence_no = Some(pair.as_str().to_string()),
            _ => (),
        };
    }
    (statement_no, sequence_no)
}

pub fn parse_60_tag(field: &Field) -> Balance {
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
    opening_balance
}

pub fn parse_61_tag(field: &Field) -> Result<StatementLine, ParseError> {
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

pub fn parse_86_tag(field: &Field) -> String {
    let parsed_field = MT940Parser::parse(Rule::tag_86_field, &field.value);
    let information_to_account_owner = parsed_field.unwrap().as_str().to_string();
    information_to_account_owner
}

pub fn parse_62_tag(field: &Field) -> Balance {
    let is_intermediate = field.tag.as_str() == "62M";
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_62_field, &field.value);
    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
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
    closing_balance
}

pub fn parse_64_tag(field: &Field) -> AvailableBalance {
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_64_field, &field.value);
    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
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
    closing_available_balance
}

pub fn parse_65_tag(field: &Field) -> AvailableBalance {
    let mut debit_credit_indicator = None;
    let mut date = None;
    let mut iso_currency_code = None;
    let mut amount = None;
    let parsed_field = MT940Parser::parse(Rule::tag_65_field, &field.value);
    let pairs = parsed_field.unwrap().next().unwrap().into_inner();
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
    forward_available_balance
}
