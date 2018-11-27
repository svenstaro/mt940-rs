//! A fast and strict MT940 parser.
//!
//! # Example
//! ```
//! extern crate mt940;
//! use mt940::parse_mt940;
//!
//! fn main() {
//!     let input = "\
//!         :20:3996-11-11111111\r\n\
//!         :25:DABADKKK/111111-11111111\r\n\
//!         :28C:00001/001\r\n\
//!         :60F:C090924EUR54484,04\r\n\
//!         :61:0909250925DR583,92NMSC1110030403010139//1234\r\n\
//!         :86:11100304030101391234\r\n\
//!         Beneficiary name\r\n\
//!         Something else\r\n\
//!         :61:0910010930DR62,60NCHGcustomer id//bank id\r\n\
//!         :86:Fees according to advice\r\n\
//!         :62F:C090930EUR53126,94\r\n\
//!         :64:C090930EUR53189,31\r\n\
//!         \r\n";
//!
//!     let input_parsed = parse_mt940(input).unwrap();
//!     assert_eq!(input_parsed[0].transaction_ref_no, "3996-11-11111111");
//! }
//! ```

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate log;

extern crate strum;
#[macro_use]
extern crate strum_macros;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate rust_decimal;

#[cfg(test)]
#[macro_use]
extern crate proptest;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
extern crate rstest;

#[cfg(test)]
extern crate regex;

mod errors;
mod tag_parsers;
mod transaction_types;
mod utils;

use chrono::prelude::*;
use pest::Parser;
use rust_decimal::Decimal;
use std::str::FromStr;

pub use errors::{ParseError, RequiredTagNotFoundError, UnexpectedTagError, VariantNotFound};
use tag_parsers::{
    parse_20_tag, parse_21_tag, parse_25_tag, parse_28c_tag, parse_60_tag, parse_61_tag,
    parse_62_tag, parse_64_tag, parse_65_tag, parse_86_tag,
};
pub use transaction_types::TransactionTypeIdentificationCode;

/// A pest parser for parsing a MT940 structure and fields.
#[derive(Parser)]
#[grammar = "mt940.pest"]
pub struct MT940Parser;

/// A single, parsed MT940 message.
///
/// Many of these might be contained in a bank statement.
///
/// For specific field documentation, see here:
/// http://www.sepaforcorporates.com/swift-for-corporates/account-statement-mt940-file-format-overview/
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Tag `:20:`
    pub transaction_ref_no: String,

    /// Tag `:21:`
    pub ref_to_related_msg: Option<String>,

    /// Tag `:25:`
    pub account_id: String,

    /// Tag `:28C:`
    pub statement_no: String,
    /// Optional part of tag `:28C:`
    pub sequence_no: Option<String>,

    /// Tag `:60F:` or `:60M:`
    ///
    /// In case this is `:60F:` it is the first opening [`Balance`].
    /// In case of `:60M:` this is the intermediate opening balance for statements following the
    /// first one.
    pub opening_balance: Balance,

    /// Tag `:61:` and `:86:`
    ///
    /// Any `:86:` preceeded by `:61:` will provide more information to that `:61:`
    pub statement_lines: Vec<StatementLine>,

    /// Tag `:62F:` or `:62M:`
    ///
    /// In case this is `:62F:` it is the first closing [`Balance`].
    /// In case of `:62M:` this is the intermediate closing balance for statements following the
    /// first one.
    pub closing_balance: Balance,

    /// Tag `:64:`
    pub closing_available_balance: Option<AvailableBalance>,

    /// Tag `:65:`
    pub forward_available_balance: Option<AvailableBalance>,

    /// Tag `:86:`
    ///
    /// A tag `:86:` not preceeded by a tag `:61` will provide information for the whole
    /// [`Message`] as opposed to just the `StatementLine`.
    pub information_to_account_owner: Option<String>,
}

/// A `StatementLine` holds information contained in tag `:61:` and tag `:86:`.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatementLine {
    pub value_date: NaiveDate,
    pub entry_date: Option<NaiveDate>,
    pub ext_debit_credit_indicator: ExtDebitOrCredit,
    pub funds_code: Option<String>,
    pub amount: Decimal,
    pub transaction_type_ident_code: TransactionTypeIdentificationCode,
    pub customer_ref: String,
    pub bank_ref: Option<String>,
    pub supplementary_details: Option<String>,
    /// This information is contained in tag `:86:`
    pub information_to_account_owner: Option<String>,
}

/// Represents a balance of an account in between statements or at the start of a statement.
///
/// The difference to [`AvailableBalance`] is that a [`Balance`] might not be final in that it might
/// have been continued from a previous bank statement. In that case, the [`Balance`] is said to be
/// intermediate. This is signaled by `is_intermediate` being set to `true`. This is generally the
/// case if this information is continued in tag `:60M:` as opposed to `:60F`.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub is_intermediate: bool,
    pub debit_credit_indicator: DebitOrCredit,
    pub date: NaiveDate,
    pub iso_currency_code: String,
    pub amount: Decimal,
}

/// Represents the currently available balance of an account.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AvailableBalance {
    pub debit_credit_indicator: DebitOrCredit,
    pub date: NaiveDate,
    pub iso_currency_code: String,
    pub amount: Decimal,
}

/// Indiciates whether a transaction was `Debit` or `Credit`.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DebitOrCredit {
    Debit,
    Credit,
}

impl FromStr for DebitOrCredit {
    type Err = VariantNotFound;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dc = if s == "C" {
            DebitOrCredit::Credit
        } else if s == "D" {
            DebitOrCredit::Debit
        } else {
            return Err(VariantNotFound(s.into()));
        };
        Ok(dc)
    }
}

/// Like [`DebitOrCredit`] with additional reverse variants.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ExtDebitOrCredit {
    Debit,
    Credit,
    ReverseDebit,
    ReverseCredit,
}

impl FromStr for ExtDebitOrCredit {
    type Err = VariantNotFound;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dc = if s == "C" {
            ExtDebitOrCredit::Credit
        } else if s == "D" {
            ExtDebitOrCredit::Debit
        } else if s == "RD" {
            ExtDebitOrCredit::ReverseCredit
        } else if s == "RC" {
            ExtDebitOrCredit::ReverseDebit
        } else {
            return Err(VariantNotFound(s.into()));
        };
        Ok(dc)
    }
}

impl Message {
    /// Construct a new [`Message`] from a list of `[Field]`s.
    ///
    /// Must start with field `:20:`. Must not contain more than one `:20:` tag.
    pub fn from_fields(fields: Vec<Field>) -> Result<Message, ParseError> {
        // Only a few tags may follow after each specific tag.
        let mut current_acceptable_tags = vec!["20"];

        let mut transaction_ref_no = None;
        let mut ref_to_related_msg = None;
        let mut account_id = None;
        let mut statement_no = None;
        let mut sequence_no = None;
        let mut opening_balance = None;
        let mut statement_lines = vec![];
        let mut closing_balance = None;
        let mut closing_available_balance = None;
        let mut forward_available_balance = None;
        let mut information_to_account_owner: Option<String> = None;

        let mut last_tag = String::default();

        for field in fields {
            debug!("Now parsing tag: {}", field.tag);

            let current_acceptable_tags_owned = current_acceptable_tags
                .iter()
                .map(|x| x.to_string())
                .collect();

            // We reject unexpected tag.
            if !current_acceptable_tags.contains(&&field.tag.as_str()) {
                return Err(UnexpectedTagError::new(
                    field.tag,
                    last_tag,
                    current_acceptable_tags_owned,
                ))?;
            }

            match field.tag.as_str() {
                "20" => {
                    transaction_ref_no = Some(parse_20_tag(&field)?);
                    current_acceptable_tags = vec!["21", "25"];
                }
                "21" => {
                    ref_to_related_msg = Some(parse_21_tag(&field)?);
                    current_acceptable_tags = vec!["25"];
                }
                "25" => {
                    account_id = Some(parse_25_tag(&field)?);
                    current_acceptable_tags = vec!["28", "28C"];
                }
                "28C" => {
                    let res = parse_28c_tag(&field)?;
                    statement_no = Some(res.0);
                    sequence_no = res.1;
                    current_acceptable_tags = vec!["60M", "60F"];
                }
                "60M" | "60F" => {
                    opening_balance = Some(parse_60_tag(&field)?);
                    current_acceptable_tags = vec!["61", "62M", "62F", "86"];
                }
                "61" => {
                    let statement_line = parse_61_tag(&field)?;
                    statement_lines.push(statement_line);
                    current_acceptable_tags = vec!["61", "86", "62M", "62F"];
                }
                "86" => {
                    let info_to_account_owner = parse_86_tag(&field)?;
                    // If the last tag was either :61: or :86: then this tag belongs to that
                    // previous tag and we'll attach the information to the previous tag.
                    match last_tag.as_str() {
                        "61" | "86" => {
                            if let Some(sl) = statement_lines.last_mut() {
                                if let Some(ref mut info) = sl.information_to_account_owner {
                                    info.push_str(&info_to_account_owner);
                                } else {
                                    sl.information_to_account_owner = Some(info_to_account_owner);
                                }
                            }
                        }
                        "62M" | "62F" | "64" | "65" => {
                            if let Some(ref mut info) = information_to_account_owner {
                                info.push_str(&info_to_account_owner);
                            } else {
                                information_to_account_owner = Some(info_to_account_owner);
                            }
                        }
                        _ => (),
                    }
                    current_acceptable_tags = vec!["61", "62M", "62F", "86"];
                }
                "62M" | "62F" => {
                    closing_balance = Some(parse_62_tag(&field)?);
                    current_acceptable_tags = vec!["64", "65", "86"];
                }
                "64" => {
                    closing_available_balance = Some(parse_64_tag(&field)?);
                    current_acceptable_tags = vec!["65", "86"];
                }
                "65" => {
                    forward_available_balance = Some(parse_65_tag(&field)?);
                    current_acceptable_tags = vec!["65", "86"];
                }
                tag => return Err(ParseError::UnknownTagError(tag.to_string())),
            }

            last_tag = field.tag;
        }

        let message = Message {
            transaction_ref_no: transaction_ref_no.ok_or_else(|| RequiredTagNotFoundError::new("20"))?,
            ref_to_related_msg,
            account_id: account_id.ok_or_else(|| RequiredTagNotFoundError::new("25"))?,
            statement_no: statement_no.ok_or_else(|| RequiredTagNotFoundError::new("28C"))?,
            sequence_no,
            opening_balance: opening_balance.ok_or_else(|| RequiredTagNotFoundError::new("60"))?,
            statement_lines,
            closing_balance: closing_balance.ok_or_else(|| RequiredTagNotFoundError::new("62"))?,
            closing_available_balance,
            forward_available_balance,
            information_to_account_owner,
        };

        Ok(message)
    }
}

/// This is a generic struct that serves as a container for the first pass of the parser.
///
/// It simply stores every field with absolutely no parsing or validation done on field values.
#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    pub tag: String,
    pub value: String,
}

impl Field {
    pub fn new(tag: &str, value: &str) -> Field {
        Field {
            tag: tag.to_string(),
            value: value.to_string(),
        }
    }
}

impl FromStr for Field {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed_field = MT940Parser::parse(Rule::field, s)?;
        let inner = parsed_field.next().unwrap().into_inner();
        let tag = inner.clone().next().unwrap().into_inner().as_str();
        let value = inner
            .clone()
            .nth(1)
            .unwrap()
            .as_str()
            .trim()
            .replace("\r\n", "\n");
        let field = Field::new(tag, &value);
        Ok(field)
    }
}

/// Parse a MT940 statement to a list of its fields.
///
/// ```ignore
/// ignored stuff in front
/// blah blah
/// :123:something
/// :456:something else
/// :789:even with
/// new line
/// like this
/// :012:and then more stuff
/// ```
///
/// The result will be a [`Vec`] of [`Field`]s.
/// There is no validation of the contents of the [`Field`]s. The contents could be nonsensical.
///
/// # Example
/// ```
/// use mt940::{parse_fields, Field};
///
/// let input = "ignored stuff in front\r\n\
///              blah blah\r\n\
///              :123:something\r\n\
///              :456:something else\r\n\
///              :789:even with\r\n\
///              new line\r\n\
///              like this\r\n\
///              :012:and then more stuff\r\n\
///              \r\n";
///
/// let expected = vec![
///     Field::new("123", "something"),
///     Field::new("456", "something else"),
///     Field::new("789", "even with\nnew line\nlike this"),
///     Field::new("012", "and then more stuff"),
/// ];
///
/// let input_parsed = parse_fields(input).unwrap();
/// assert_eq!(expected, input_parsed);
/// ```
pub fn parse_fields(statement: &str) -> Result<Vec<Field>, pest::error::Error<Rule>> {
    let parsed_fields = MT940Parser::parse(Rule::fields, statement)?;

    let mut fields = vec![];
    for parsed_field in parsed_fields {
        if let Rule::EOI = parsed_field.as_rule() {
            break;
        }
        let inner = parsed_field.into_inner();
        let tag = inner.clone().next().unwrap().into_inner().as_str();
        let value = inner
            .clone()
            .nth(1)
            .unwrap()
            .as_str()
            .trim()
            .replace("\r\n", "\n");
        let field = Field::new(tag, &value);
        fields.push(field);
    }

    Ok(fields)
}

/// Parse and validate a MT940 statement.
///
/// Result will be a [`Vec`] of all contained [`Message`]s.
///
/// # Example
/// ```
/// # extern crate mt940;
/// # extern crate chrono;
/// # extern crate rust_decimal;
/// # use chrono::prelude::*;
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
/// # use mt940::{Message, AvailableBalance, Balance, StatementLine};
/// # use mt940::{DebitOrCredit, ExtDebitOrCredit, TransactionTypeIdentificationCode};
/// use mt940::parse_mt940;
///
/// let input = "\
///     :20:3996-11-11111111\r\n\
///     :25:DABADKKK/111111-11111111\r\n\
///     :28C:00001/001\r\n\
///     :60F:C090924EUR54484,04\r\n\
///     :61:0909250925DR583,92NMSC1110030403010139//1234\r\n\
///     :86:11100304030101391234\r\n\
///     Beneficiary name\r\n\
///     Something else\r\n\
///     :61:0910010930DR62,60NCHGcustomer id//bank id\r\n\
///     :86:Fees according to advice\r\n\
///     :62F:C090930EUR53126,94\r\n\
///     :64:C090930EUR53189,31\r\n\
///     \r\n";
///
/// let expected = vec![Message {
///     transaction_ref_no: "3996-11-11111111".to_string(),
///     ref_to_related_msg: None,
///     account_id: "DABADKKK/111111-11111111".to_string(),
///     statement_no: "00001".to_string(),
///     sequence_no: Some("001".to_string()),
///     opening_balance: Balance {
///         is_intermediate: false,
///         debit_credit_indicator: DebitOrCredit::Credit,
///         date: NaiveDate::from_ymd(2009, 09, 24),
///         iso_currency_code: "EUR".to_string(),
///         amount: Decimal::from_str("54484.04").unwrap(),
///     },
///     statement_lines: vec![
///         StatementLine {
///             value_date: NaiveDate::from_ymd(2009, 09, 25),
///             entry_date: Some(NaiveDate::from_ymd(2009, 09, 25)),
///             ext_debit_credit_indicator: ExtDebitOrCredit::Debit,
///             funds_code: Some("R".to_string()),
///             amount: Decimal::from_str("583.92").unwrap(),
///             transaction_type_ident_code: TransactionTypeIdentificationCode::MSC,
///             customer_ref: "1110030403010139".to_string(),
///             bank_ref: Some("1234".to_string()),
///             supplementary_details: None,
///             information_to_account_owner: Some(
///                 "11100304030101391234\nBeneficiary name\nSomething else".to_string(),
///             ),
///         },
///         StatementLine {
///             value_date: NaiveDate::from_ymd(2009, 10, 01),
///             entry_date: Some(NaiveDate::from_ymd(2009, 09, 30)),
///             ext_debit_credit_indicator: ExtDebitOrCredit::Debit,
///             funds_code: Some("R".to_string()),
///             amount: Decimal::from_str("62.60").unwrap(),
///             transaction_type_ident_code: TransactionTypeIdentificationCode::CHG,
///             customer_ref: "customer id".to_string(),
///             bank_ref: Some("bank id".to_string()),
///             supplementary_details: None,
///             information_to_account_owner: Some("Fees according to advice".to_string()),
///         },
///     ],
///     closing_balance: Balance {
///         is_intermediate: false,
///         debit_credit_indicator: DebitOrCredit::Credit,
///         date: NaiveDate::from_ymd(2009, 09, 30),
///         iso_currency_code: "EUR".to_string(),
///         amount: Decimal::from_str("53126.94").unwrap(),
///     },
///     closing_available_balance: Some(AvailableBalance {
///         debit_credit_indicator: DebitOrCredit::Credit,
///         date: NaiveDate::from_ymd(2009, 09, 30),
///         iso_currency_code: "EUR".to_string(),
///         amount: Decimal::from_str("53189.31").unwrap(),
///     }),
///     forward_available_balance: None,
///     information_to_account_owner: None,
/// }];
/// let input_parsed = parse_mt940(input).unwrap();
/// assert_eq!(expected, input_parsed);
/// ```
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
    use regex::Regex;

    use super::*;

    #[test]
    fn parse_mt940_fields() {
        let input = "ignored stuff in front
                     blah blah
                     :123:something\r\n\
                     :456:something else\r\n\
                     :789:even with\r\n\
                     new line\r\n\
                     like this\r\n\
                     :012:and then more stuff\r\n\
                     \r\n";

        let expected = vec![
            Field::new("123", "something"),
            Field::new("456", "something else"),
            Field::new("789", "even with\nnew line\nlike this"),
            Field::new("012", "and then more stuff"),
        ];

        let input_parsed = parse_fields(input).unwrap();
        assert_eq!(expected, input_parsed);
    }

    proptest! {
        #[test]
        fn dont_crash(tag in "[[:alnum:]]+", value in r"[0-9A-Za-z/\-\?:\(\)\.,‘\+\{\} ]+") {
            let re_tag_like = Regex::new(":.*:").unwrap();
            prop_assume!(!re_tag_like.is_match(&value), "Can't have a value that looks like a tag");

            // I know this is pretty arbitrary but I think it's a reasonable assumption to make.
            // I don't think banks encode information in leading or trailing whitespace considering
            // these formats are made for print.
            let re_no_ws_in_front_or_end = Regex::new(r"^[^\s]+(\s+[^\s]+)*$").unwrap();
            prop_assume!(re_no_ws_in_front_or_end.is_match(&value), "Can't have a value that has whitespace in front or end");

            let parsed = parse_fields(&format!(":{}:{}", tag, value)).unwrap();
            prop_assert_eq!((&parsed[0].tag, &parsed[0].value), (&tag, &value));
        }
    }
}
