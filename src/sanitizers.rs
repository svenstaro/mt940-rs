//! This module contains a collection of sanitizers which is really just a fancy way of saying
//! that this is a bunch of functions which take strings, change them, and give them back.

use deunicode::deunicode_char;
use pest::Parser;

use crate::MT940Parser;
use crate::Rule;

/// Run all sanitizers on the input in a useful order.
///
/// If you don't really care exactly _how_ you're input is sanitized and just want it to work, this
/// is probably the function to use. Be aware that it's possible that some data could be truncated
/// in order to make valid statements.
pub fn sanitize(s: &str) -> String {
    let s1 = to_swift_charset(s);
    let s2 = strip_stuff_between_messages(&s1);
    strip_excess_tag86_lines(&s2)
}

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
                let deunicoded = if x == 'ä' {
                    // Due to https://github.com/kornelski/deunicode/issues/15, we'll deunicode 'ä'
                    // ourselves.
                    "a".to_string()
                } else {
                    deunicode_char(x).unwrap_or(".").to_string()
                };
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

/// Remove stuff between messages.
///
/// Sometimes, statements will have messages separated with `-` or the like to keep the visually
/// seperate like this:
///
/// ```ignore
/// ...
/// :62F:D070904EUR1237628,23
/// :64:D070904EUR1237628,23
/// -
/// :20:T089413956000001
/// :25:50880050/0194777100888
/// ...
/// ```
///
/// However, this makes them uncompliant.
///
/// This sanitizer gets rid of that.
pub fn strip_stuff_between_messages(s: &str) -> String {
    // Find all :20: tags and remove any non-tag lines before those.
    let total_lines = s.lines().count();
    let mut lines_with_tag_20 = vec![];
    let mut lines_with_tags = vec![];
    let mut last_tag = "20";

    // Do one pass to find all the indices of tag 20 and non-tag 20 lines.
    for (i, line) in s.lines().enumerate() {
        let parsed = MT940Parser::parse(Rule::field, line);
        if let Ok(mut parsed) = parsed {
            last_tag = parsed
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str();
            if last_tag == "20" {
                lines_with_tag_20.push(i);
            }
            lines_with_tags.push(i);
        }
    }

    let mut lines_to_delete = vec![];

    // Do a second pass to figure out which lines we don't want.
    for tag_20_index in lines_with_tag_20 {
        // From the tag 20 index, travel upwards until either hitting another tag's index or
        // until hitting index 0.
        let mut i = tag_20_index;
        while i > 0 {
            i -= 1;

            // Terminate on the first line that contains another tag.
            if lines_with_tags.contains(&i) {
                break;
            } else {
                lines_to_delete.push(i);
            }
        }
    }

    // There is a special case to handle for the end of the file:
    // If the very last tag is a tag 86 then we'll allow any non-tag lines after it towards the end
    // of the file. However, if the last tag is a non-86-tag then we'll remove any additional lines
    // up to the last tag.
    if last_tag != "86" {
        let last_tag_index = *lines_with_tags.last().unwrap_or(&0) + 1;
        lines_to_delete.extend(last_tag_index..total_lines);
    }

    // Do a third pass to actually copy only the wanted lines from the input to the output.
    s.lines()
        .enumerate()
        .filter(|&(i, _contents)| (!lines_to_delete.contains(&i)))
        .map(|(_i, contents)| contents)
        .chain(std::iter::once(""))
        .collect::<Vec<&str>>()
        .join("\r\n")
}

/// Remove excess lines on tag 86 statements beyond the 6 allowed.
///
/// Note that you potentially lose information with this sanitizer.
pub fn strip_excess_tag86_lines(input: &str) -> String {
    let mut lines_to_delete = vec![];

    // Get a list of lines where tag 86 messages start.
    let tag_86_lines = input.lines().enumerate().filter_map(|(line, contents)| {
        if contents.starts_with(":86:") {
            Some(line)
        } else {
            None
        }
    });

    for line_no in tag_86_lines {
        let lines = input.lines().skip(line_no + 1);

        // Find all lines excess of the 5 allowed additional lines (6 in total counting the skipped line above).
        let to_delete = lines
            .enumerate()
            .take_while(|(_, contents)| !contents.starts_with(':'))
            .filter_map(move |(line, _)| {
                if line >= 5 {
                    Some(line + line_no + 1)
                } else {
                    None
                }
            });

        lines_to_delete.extend(to_delete);
    }

    input
        .lines()
        .enumerate()
        .filter(|&(line, _contents)| (!lines_to_delete.contains(&line)))
        .map(|(_line, contents)| contents)
        .chain(std::iter::once(""))
        .collect::<Vec<&str>>()
        .join("\r\n")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proptest::{prop_assert, proptest};
    use rstest::rstest;

    use super::*;

    proptest! {
        #[test]
        fn to_swift_charset_no_parsing_failure_after_conversion(input in r".+") {
            let result = to_swift_charset(&input);
            let parsed = MT940Parser::parse(Rule::swift_chars, &result);
            prop_assert!(parsed.is_ok());
        }
    }

    #[test]
    fn to_swift_charset_sanitize_sentence() {
        let input = "hällö waß íst lös";
        let result = to_swift_charset(input);
        let expected = "hallo wass ist los";
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("ä", "a")]
    #[case("ö", "o")]
    #[case("ú", "u")]
    #[case("é", "e")]
    #[case("å", "a")]
    #[case("á", "a")]
    #[case("ß", "ss")]
    #[case("ú", "u")]
    #[case("ó", "o")]
    #[case("í", "i")]
    #[case("ë", "e")]
    #[case("=", ".")]
    #[case("!", ".")]
    fn to_swift_charset_special_char_conversions(#[case] input: &str, #[case] expected: &str) {
        let result = to_swift_charset(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn strip_stuff_between_messages_success() {
        let input = "\
            :86:asdasdads\r\n\
            ------\r\n\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            here is ok\r\n\
            :64:end of message\r\n\
            stuff between messages\r\n\
            should be removed\r\n\
            :20:aaaaa\r\n\
            :64:some more\r\n\
            ö»»«»«äää\r\n\
            :20:lolab\r\n\
            :86:zzzz\r\n\
            :64:asda\r\n\
            --\r\n\
        ";
        let expected = "\
                        :86:asdasdads\r\n\
                        :20:vvvvv\r\n\
                        :86:hello\r\n\
                        multi line string\r\n\
                        here is ok\r\n\
                        :64:end of message\r\n\
                        :20:aaaaa\r\n\
                        :64:some more\r\n\
                        :20:lolab\r\n\
                        :86:zzzz\r\n\
                        :64:asda\r\n\
                        ";
        let result = strip_stuff_between_messages(input);
        assert_eq!(result, expected);
    }

    /// Last lines in the file will be stripped if last tag is not tag 86.
    /// Tag 86 is a multiline tag and can validly be placed at the end of a message.
    #[test]
    fn strip_stuff_between_messages_last_is_86() {
        let input = "\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            here is ok\r\n\
            --\r\n\
        ";
        let expected = "\
                        :20:vvvvv\r\n\
                        :86:hello\r\n\
                        multi line string\r\n\
                        here is ok\r\n\
                        --\r\n\
                        ";
        let result = strip_stuff_between_messages(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn excess_tag86_are_stripped() {
        let input = "\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            here is ok even with date that looks like a tag 20:10:43\r\n\
            but not when\r\n\
            it is way too many\r\n\
            lines\r\n\
            in fact i shouldnt be here\r\n\
            and i shouldnt either\r\n\
            :62F:C123EUR321,98\r\n\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            but not many lines\r\n\
            :62F:C123EUR321,98\r\n\
            :20:vvvvv\r\n\
            :86:hi there\r\n\
            a very multi lined string\r\n\
            here is ok even with date that looks like a tag 20:86:43\r\n\
            but not when\r\n\
            it is way too many\r\n\
            lines\r\n\
            in fact i shouldnt be here\r\n\
            and i shouldnt either\r\n\
            and i certainly aint supposed to be here as well\r\n\
            :62F:C321EUR123,98\r\n\
        ";
        let expected = "\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            here is ok even with date that looks like a tag 20:10:43\r\n\
            but not when\r\n\
            it is way too many\r\n\
            lines\r\n\
            :62F:C123EUR321,98\r\n\
            :20:vvvvv\r\n\
            :86:hello\r\n\
            multi line string\r\n\
            but not many lines\r\n\
            :62F:C123EUR321,98\r\n\
            :20:vvvvv\r\n\
            :86:hi there\r\n\
            a very multi lined string\r\n\
            here is ok even with date that looks like a tag 20:86:43\r\n\
            but not when\r\n\
            it is way too many\r\n\
            lines\r\n\
            :62F:C321EUR123,98\r\n\
        ";
        let result = strip_excess_tag86_lines(input);
        assert_eq!(result, expected);
    }
}
