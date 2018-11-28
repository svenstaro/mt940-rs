//! This module contains a collection of sanitizers which is really just a fancy way of saying
//! that this is a bunch of functions which take strings, change them, and give them back.

use deunicode::deunicode_char;
use pest::Parser;

use MT940Parser;
use Rule;

/// Try to make a given input conform to the SWIFT MT101 allowed charset.
///
/// This works by running `deunicode_char` on all non-SWIFT characters. That gets rid of characters
/// like 'ä', 'ö', 'ü', 'ú' and so and converts them into their sensible ASCII equivalents.
/// Any remaining non-SWIFT characters (like '!', '=', etc) will be replaced with a dot ('.') each.
/// [SWIFT MT101 characters reference here](http://www.sepaforcorporates.com/swift-for-corporates/quick-guide-swift-mt101-format/).
pub fn to_swift_charset(s: &str) -> String {
    // Parse the string char by char and see whether it's a conforming swift char. If it isn't,
    // we'll want to run deunicode.
    s.chars()
        .map(|x| {
            let char_as_string = x.to_string();
            let parsed = MT940Parser::parse(Rule::swift_char, &char_as_string);
            // If parsing succeeds, we already have a SWIFT charset allowable character, yay!
            // However, if it doesn't, we'll have to be sensible and smart about it...
            if parsed.is_ok() {
                char_as_string.clone()
            } else {
                // This is the first attempt to make a non-SWIFT character into an allowed character.
                let deunicoded = deunicode_char(x).unwrap_or(".").to_string();
                // Also note that we have to use the `Rule::swift_chars` here because a single
                // Unicode character might be deunicoded to multiple ASCII chars!
                let parsed_after_deunicode = MT940Parser::parse(Rule::swift_chars, &deunicoded);
                if parsed_after_deunicode.is_ok() {
                    deunicoded.clone()
                } else {
                    // If all else fails, we can only replace this character with a dot and move
                    // on.
                    ".".to_string()
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest_parametrize;

    use super::*;

    proptest! {
        #[test]
        fn no_parsing_failure_after_conversion(input in r".+") {
            let result = to_swift_charset(&input);
            let parsed = MT940Parser::parse(Rule::swift_chars, &result);
            prop_assert!(parsed.is_ok());
        }
    }

    #[test]
    fn sanitize_sentence() {
        let input = "hällö waß íst lös";
        let result = to_swift_charset(input);
        let expected = "hallo wass ist los";
        assert_eq!(result, expected);
    }

    #[rstest_parametrize(
        input,
        expected,
        case("ä", "a"),
        case("ö", "o"),
        case("ú", "u"),
        case("é", "e"),
        case("å", "a"),
        case("á", "a"),
        case("ß", "ss"),
        case("ú", "u"),
        case("ó", "o"),
        case("í", "i"),
        case("ë", "e"),
        case("=", "."),
        case("!", ".")
    )]
    fn special_char_conversions(input: &str, expected: &str) {
        let result = to_swift_charset(input);
        assert_eq!(result, expected);
    }
}
