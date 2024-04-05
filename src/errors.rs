use thiserror::Error;

use crate::Rule;

#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum DateParseError {
    #[error("Date parsing failed for date: '{}-{}-{}'", year, month, day)]
    OutOfRange {
        year: String,
        month: String,
        day: String,
    },

    #[error("Pest parsing error: {}", _0)]
    PestParseError(Box<pest::error::Error<Rule>>),
}

impl From<pest::error::Error<Rule>> for DateParseError {
    fn from(err: pest::error::Error<Rule>) -> DateParseError {
        DateParseError::PestParseError(Box::new(err))
    }
}

/// Error thrown if a variant for an enum can't be found.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("Variant not found: {}", _0)]
pub struct VariantNotFound(pub String);

/// Error thrown when parsing of a MT940 amount fails.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum AmountParseError {
    #[error("Too many commas in amount: '{}'", _0)]
    TooManyCommas(String),

    #[error("No comma found in amount: '{}'", _0)]
    NoComma(String),

    #[error("Couldn't parse as integer: '{}'", _0)]
    IntParseError(std::num::ParseIntError),
}

/// Error thrown when parsing fails.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum ParseError {
    #[error("Pest parsing error: {}", _0)]
    PestParseError(Box<pest::error::Error<Rule>>),

    #[error("{}", _0)]
    UnexpectedTagError(UnexpectedTagError),

    #[error("{}", _0)]
    DateParseError(Box<DateParseError>),

    #[error("{}", _0)]
    RequiredTagNotFoundError(RequiredTagNotFoundError),

    #[error("Unknown tag: '{}'", _0)]
    UnknownTagError(String),

    #[error("{}", _0)]
    VariantNotFound(VariantNotFound),

    #[error("{}", _0)]
    AmountParseError(AmountParseError),
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> ParseError {
        ParseError::PestParseError(Box::new(err))
    }
}

impl From<Box<pest::error::Error<Rule>>> for ParseError {
    fn from(err: Box<pest::error::Error<Rule>>) -> ParseError {
        ParseError::PestParseError(err)
    }
}

impl From<DateParseError> for ParseError {
    fn from(err: DateParseError) -> ParseError {
        ParseError::DateParseError(Box::new(err))
    }
}

impl From<UnexpectedTagError> for ParseError {
    fn from(err: UnexpectedTagError) -> ParseError {
        ParseError::UnexpectedTagError(err)
    }
}

impl From<RequiredTagNotFoundError> for ParseError {
    fn from(err: RequiredTagNotFoundError) -> ParseError {
        ParseError::RequiredTagNotFoundError(err)
    }
}

impl From<VariantNotFound> for ParseError {
    fn from(err: VariantNotFound) -> ParseError {
        ParseError::VariantNotFound(err)
    }
}

impl From<AmountParseError> for ParseError {
    fn from(err: AmountParseError) -> ParseError {
        ParseError::AmountParseError(err)
    }
}

/// Error thrown when an unexpected tag was found.
///
/// Some tags must never follow other tags. If that happens for some reason, we can safely assume
/// that the input data is faulty.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error(
    "Unexpected tag '{}' found. Expected one of '{:?}'. The tag before this one was '{}'.",
    current_tag,
    expected_tags,
    last_tag
)]
pub struct UnexpectedTagError {
    current_tag: String,
    last_tag: String,
    expected_tags: Vec<String>,
}

impl UnexpectedTagError {
    pub fn new(
        current_tag: &str,
        last_tag: &str,
        expected_tags: Vec<String>,
    ) -> UnexpectedTagError {
        UnexpectedTagError {
            current_tag: current_tag.to_string(),
            last_tag: last_tag.to_string(),
            expected_tags,
        }
    }
}

/// Error thrown if a required tag was not found.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
#[error("Required tag '{}' not found.", required_tag)]
pub struct RequiredTagNotFoundError {
    required_tag: String,
}

impl RequiredTagNotFoundError {
    pub fn new(tag: &str) -> RequiredTagNotFoundError {
        RequiredTagNotFoundError {
            required_tag: tag.to_string(),
        }
    }
}
