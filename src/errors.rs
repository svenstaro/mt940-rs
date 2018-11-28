use std::error;
use std::fmt;

use Rule;

/// Error thrown if a variant for an enum can't be found.
#[derive(Debug)]
pub struct VariantNotFound(pub String);

/// Error thrown when parsing of a MT940 amount fails.
#[derive(Debug)]
pub enum AmountParseError {
    TooManyCommas,
    NoComma,
    IntParseError(std::num::ParseIntError),
}

/// Error thrown when parsing fails.
#[derive(Debug)]
pub enum ParseError {
    PestParseError(pest::error::Error<Rule>),
    UnexpectedTagError(UnexpectedTagError),
    RequiredTagNotFoundError(RequiredTagNotFoundError),
    UnknownTagError(String),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(error::Error + 'static)> {
        match *self {
            ParseError::PestParseError(ref err) => Some(err),
            ParseError::UnexpectedTagError(ref err) => Some(err),
            ParseError::RequiredTagNotFoundError(ref err) => Some(err),
            ParseError::UnknownTagError(ref _err) => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::PestParseError(ref err) => err.fmt(f),
            ParseError::UnexpectedTagError(ref err) => err.fmt(f),
            ParseError::RequiredTagNotFoundError(ref err) => err.fmt(f),
            ParseError::UnknownTagError(ref err) => write!(f, "Unknown Tag '{}'", err),
        }
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> ParseError {
        ParseError::PestParseError(err)
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

/// Error thrown when an unexpected tag was found.
///
/// Some tags must never follow other tags. If that happens for some reason, we can safely assume
/// that the input data is faulty.
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

/// Error thrown if a required tag was not found.
#[derive(Debug)]
pub struct RequiredTagNotFoundError {
    required_tag: String,
}

impl RequiredTagNotFoundError {
    pub fn new(required_tag: &str) -> RequiredTagNotFoundError {
        RequiredTagNotFoundError {
            required_tag: required_tag.to_string(),
        }
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
