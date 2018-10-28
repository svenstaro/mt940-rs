extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate derive_builder;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use pest::Parser;

#[derive(Parser)]
#[grammar = "mt940.pest"]
pub struct MT940Parser;

#[derive(Debug, PartialEq, Builder)]
pub struct Message {
    pub transaction_ref_no: String,
    #[builder(default)]
    pub ref_to_related_msg: Option<String>,
    pub account_id: String,
    // pub statement_no: u32,
    // pub sequence_no: u32,
    // pub opening_balance: u32,
}

fn parse_mt940(statement: &str) -> Result<Vec<Message>, pest::error::Error<Rule>> {
    let parsed_statement = MT940Parser::parse(Rule::statement, statement)?;
    let mut messages = vec![];
    for parsed_message in parsed_statement {
        if let Rule::EOI = parsed_message.as_rule() {
            break;
        }

        let mut m_builder = MessageBuilder::default();
        let mut field_pairs = parsed_message.clone().into_inner();
        for field_pair in field_pairs {
            println!("{:#?}", field_pair);
            match field_pair.as_rule() {
                Rule::field_20_entry => {
                    m_builder.transaction_ref_no(
                        field_pair
                            .clone()
                            .into_inner()
                            .skip(1)
                            .next()
                            .unwrap()
                            .as_str()
                            .to_owned(),
                    );
                }
                Rule::field_21_entry => {
                    m_builder.ref_to_related_msg(Some(
                        field_pair
                            .clone()
                            .into_inner()
                            .skip(1)
                            .next()
                            .unwrap()
                            .as_str()
                            .to_owned(),
                    ));
                }
                Rule::field_25_entry => {
                    m_builder.account_id(
                        field_pair
                            .clone()
                            .into_inner()
                            .skip(1)
                            .next()
                            .unwrap()
                            .as_str()
                            .to_owned(),
                    );
                }
                _ => (),
            };
        }
        let message = m_builder.build();
        match message {
            Ok(m) => messages.push(m),
            Err(e) => panic!(e),
        }
        // println!("{:#?}", field_pairs.as_rule());
        //     let tag = inner_record
        //         .next()
        //         .unwrap()
        //         .clone()
        //         .into_inner()
        //         .next()
        //         .unwrap()
        //         .as_str()
        //         .to_owned();
        //     let message = inner_record
        //         .as_str()
        //         .replace("\r\n", "\n")
        //         .trim()
        //         .to_owned();
        // messages.push(message);
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
        let test_data = "\
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
}
