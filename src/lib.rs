extern crate pest;
#[macro_use]
extern crate pest_derive;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use pest::Parser;

#[derive(Parser)]
#[grammar = "mt940.pest"]
pub struct MT940Parser;

#[derive(Debug, Eq, PartialEq)]
pub struct Message {
    pub transaction_ref_no: String,
    pub ref_to_related_msg: Option<String>,
    pub account_id: String,
    // pub statement_no: u32,
    // pub sequence_no: u32,
    // pub opening_balance: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    pub tag: String,
    pub value: String,
}

pub fn parse_fields(statement: &str) -> Result<Vec<Field>, pest::error::Error<Rule>> {
    let parsed_fields = MT940Parser::parse(Rule::fields, statement)?;

    let mut fields = vec![];
    for parsed_field in parsed_fields {
        if let Rule::EOI = parsed_field.as_rule() {
            break;
        }
        let inner = parsed_field.into_inner();
        let tag = inner
            .clone()
            .next()
            .unwrap()
            .into_inner()
            .as_str()
            .to_string();
        let value = inner
            .clone()
            .skip(1)
            .next()
            .unwrap()
            .as_str()
            .trim()
            .replace("\r\n", "\n")
            .to_string();
        let field = Field { tag, value };
        fields.push(field);
    }

    Ok(fields)
}

fn parse_mt940(statement: &str) -> Result<Vec<Message>, String> {
    let fields = parse_fields(statement).map_err(|e| format!("{}", e))?;

    println!("{:#?}", fields);

    let mut messages = vec![];
    let mut current_transaction_ref_no = "";

    // Only a few tags may follow after each specific tag.
    let mut next_expected_tags = vec!["20"];

    for field in fields {
        // TODO Log crate
        println!("current tag: {}", field.tag);
        if !next_expected_tags.contains(&&field.tag.as_str()) {
            // TODO: Custom Error type
            return Err(format!(
                "Expected one of {expected:?}, instead found {found}",
                expected = next_expected_tags,
                found = field.tag
            ));
        }



        next_expected_tags = match field.tag.as_str() {
            "20" => vec!["21", "25"],
            "21" => vec!["25"],
            "25" => vec!["28", "28C"],
            "28" | "28C" => vec!["60M", "60F"],
            "60M" | "60F" => vec!["61", "62M", "62F", "86"],
            "61" => vec!["86", "62M", "62F"],
            "86" => vec!["61", "62M", "62F"],
            "62M" | "62F" => vec!["64", "65", "86"],
            "64" => vec!["65", "86"],
            "65" => vec!["65", "86"],
            "86" => vec![],
            _ => unreachable!(),
        };
        println!("{:#?}", next_expected_tags);
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
