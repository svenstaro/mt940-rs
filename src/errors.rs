use std::error;
use std::fmt;

use Rule;

#[derive(Debug)]
pub enum AmountParseError {
    TooManyCommas,
    NoComma,
    IntParseError(std::num::ParseIntError),
}

#[derive(Debug)]
pub enum ParseError {
    PestParseError(pest::error::Error<Rule>),
    UnexpectedTagError(UnexpectedTagError),
    RequiredTagNotFoundError(RequiredTagNotFoundError),
    InvalidTransactionIdentCode(String),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(error::Error + 'static)> {
         match *self {
             ParseError::PestParseError(ref err) => Some(err),
             ParseError::UnexpectedTagError(ref err) => Some(err),
             ParseError::RequiredTagNotFoundError(ref err) => Some(err),
             ParseError::InvalidTransactionIdentCode(ref _err) => None,
         }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::PestParseError(ref err) => err.fmt(f),
            ParseError::UnexpectedTagError(ref err) => err.fmt(f),
            ParseError::RequiredTagNotFoundError(ref err) => err.fmt(f),
            ParseError::InvalidTransactionIdentCode(ref err) => write!(f, "Invalid Transaction Type Identification Code '{}'", err),
        }
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

#[derive(Debug)]
pub struct UnexpectedTagError {
    current_tag: String,
    last_tag: String,
    expected_tags: Vec<String>,
}

impl UnexpectedTagError {
    pub fn new(
        current_tag: String,
        last_tag: String,
        expected_tags: Vec<String>,
    ) -> UnexpectedTagError {
        UnexpectedTagError {
            current_tag,
            last_tag,
            expected_tags,
        }
    }
}

impl error::Error for UnexpectedTagError {
    fn source(&self) -> Option<&(error::Error + 'static)> {
        None
    }
}

impl fmt::Display for UnexpectedTagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unexpected tag '{}' found. Expected one of '{}'. The tag before this one was '{:?}.",
            self.current_tag, self.last_tag, self.expected_tags
        )
    }
}

#[derive(Debug)]
pub struct RequiredTagNotFoundError {
    required_tag: String,
}

impl RequiredTagNotFoundError {
    pub fn new(required_tag: String) -> RequiredTagNotFoundError {
        RequiredTagNotFoundError { required_tag }
    }
}

impl error::Error for RequiredTagNotFoundError {
    fn source(&self) -> Option<&(error::Error + 'static)> {
        None
    }
}

impl fmt::Display for RequiredTagNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Required tag '{}' not found.", self.required_tag,)
    }
}
