extern crate combine;

use combine::parser::byte;
use combine::parser::char;
use combine::parser::repeat::take_until;
use combine::*;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub tag: String,
    pub message: String,
}

fn mt940_tag<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        token(':'),
        char::digit(),
        many::<String, _>(char::alpha_num()),
        token(':'),
    )
        .map(|(_, x, y, _)| format!("{}{}", x, y))
}

fn mt940_record_start<I>() -> impl Parser<Input = I, Output = (String, String)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (mt940_tag(), take_until(byte::bytes(&b"\r\n"[..])))
}

// fn mt940_statement<I>() -> impl Parser<Input = I, Output = (String)>
// where
//     I: Stream<Item = char>,
//     I::Error: ParseError<I::Item, I::Range, I::Position>,
// {
//     let record_start = (mt940_tag, take_until(byte::bytes(&b"\r\n"[..])));
//
//     (
//         record_start,
//         many((not_followed_by(look_ahead(mt940_tag)), "\r\n")),
//     )
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mt940_tag() {
        let result = mt940_tag().parse(":20:");
        assert_eq!(result, Ok(("20".to_string(), "")));
    }

    // #[test]
    // fn parse_mt940_message() {
    //     assert_eq!(
    //         mt940_message("3996-11-11111111\r\n".into()),
    //         Ok((EMPTY, "3996-11-11111111".into()))
    //     );
    // }
    //
    // #[test]
    // fn parse_mt940_record() {
    //     assert_eq!(
    //         mt940_record(":20:3996-11-11111111\r\n".into()),
    //         Ok((
    //             EMPTY,
    //             Statement {
    //                 prefix: "20".to_string(),
    //                 message: "3996-11-11111111".to_string(),
    //             }
    //         ))
    //     );
    // }
    //
    // #[test]
    // fn parse_mt940_statement() {
    //     let test_data = ":20:3996-1234567890\r\n:25:DABADKKK/1234567890\r\n";
    //     // let test_data = b":20:3996-1234567890\r\n:25:DABADKKK/1234567890\r\n";
    //     // let test_data = b"\
    //     // :20:3996-1234567890\r\n\
    //     // :25:DABADKKK/1234567890\r\n\
    //     // :28C:00014/001\r\n\
    //     // :60F:C091019DKK3859701,48\r\n\
    //     // :86:For your inform. IBAN no.: DK5030001234567890\r\n\
    //     // :86:DABADKKK                                                 \r\n\
    //     // :86:1234567890\r\n\
    //     // :86:DANSKE BANK                        HOLMENS KANAL 2-12\r\n\
    //     // :61:0910201020DK5312,50NMSCDBT.teste kunden\r\n\
    //     // :86:F.M.T.\r\n\
    //     // V/TESTE KUNDEN\r\n\
    //     // HOLMENS KANAL 2-12\r\n\
    //     // 1192  KOBENHAVN H\r\n\
    //     // :61:0910201020DK3009,51NMSCDBT.Pet Van Park\r\n\
    //     // :86:DBTS 1111272333/Bnf. PET VAN PARK AMSTERDAM/Bnf.acc. NL47ABNAXXXX\r\n\
    //     // 558756/Our fee DKK 40,00/Foreign fee DKK 200,00\r\n\
    //     // :62F:C091020DKK3851379,47\r\n\
    //     // :64:C091020DKK3851379,47\r\n\
    //     // \r\n";
    //
    //     let expected = vec![
    //         Statement {
    //             prefix: "20".to_string(),
    //             message: "3996-1234567890".to_string(),
    //         },
    //         Statement {
    //             prefix: "25".to_string(),
    //             message: "DABADKKK/1234567890".to_string(),
    //         },
    //         // Statement {
    //         //     prefix: "28C".to_string(),
    //         //     message: "00014/001".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "60F".to_string(),
    //         //     message: "C091019DKK3859701,48".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "86".to_string(),
    //         //     message: ")
    //         //         For your inform. IBAN no.: DK5030001234567890\n\
    //         //         DABADKKK\n\
    //         //         1234567890\n\
    //         //         DANSKE BANK                        HOLMENS KANAL 2-12\n\
    //         //     ".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "61".to_string(),
    //         //     message: "0910201020DK5312,50NMSCDBT.teste kunden".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "86".to_string(),
    //         //     message: "F.M.T.\n\
    //         //               V/TESTE KUNDEN\n\
    //         //               HOLMENS KANAL 2-12\n\
    //         //               1192  KOBENHAVN H\n\
    //         //               ".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "61".to_string(),
    //         //     message: "0910201020DK3009,51NMSCDBT.Pet Van Park".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "86".to_string(),
    //         //     message: "DBTS 1111272333/Bnf. PET VAN PARK AMSTERDAM/Bnf.acc. NL47ABNAXXXX\n\
    //         //               558756/Our fee DKK 40,00/Foreign fee DKK 200,00"
    //         //         .to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "62F".to_string(),
    //         //     message: "C091020DKK3851379,47".to_string(),
    //         // },
    //         // Statement {
    //         //     prefix: "64".to_string(),
    //         //     message: "C091020DKK3851379,47".to_string(),
    //         // },
    //     ];
    //     assert_eq!(mt940_statement(test_data.into()), Ok((EMPTY, expected)));
    // }
}
