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

#[derive(Debug, PartialEq)]
pub struct Record {
    pub tag: String,
    pub message: String,
}

fn parse_mt940(statement: &str) -> Result<Vec<Record>, pest::error::Error<Rule>> {
    let parse_result = MT940Parser::parse(Rule::statement, statement)?;
    let mut records = vec![];
    for record in parse_result {
        if let Rule::record = record.as_rule() {
            let mut inner_record = record.into_inner();
            let tag = inner_record
                .next()
                .unwrap()
                .clone()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_owned();
            let message = inner_record
                .as_str()
                .replace("\r\n", "\n")
                .trim()
                .to_owned();
            records.push(Record { tag, message });
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mt940_tag() {
        let expected = "20";
        let result = MT940Parser::parse(Rule::tag, ":20:");
        assert_eq!(
            expected,
            result.unwrap().next().unwrap().into_inner().as_str()
        );
    }

    #[test]
    fn parse_mt940_record_single_line() {
        let expected = Record {
            tag: "20".to_string(),
            message: "3996-11-11111111".to_string(),
        };
        let result = parse_mt940(":20:3996-11-11111111\r\n").unwrap();
        assert_eq!(expected, result[0]);
    }

    #[test]
    fn parse_mt940_record() {
        let expected = Record {
            tag: "20".to_string(),
            message: "3996-11-11111111\nTES TTEST\nMORETEST".to_string(),
        };
        let result = parse_mt940(
            ":20:3996-11-11111111\r\nTES TTEST\r\nMORETEST\r\n:50:some-other-message\r\n",
        )
        .unwrap();
        assert_eq!(expected, result[0]);
    }

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

        let expected = vec![
            Record {
                tag: "20".to_string(),
                message: "3996-1234567890".to_string(),
            },
            Record {
                tag: "25".to_string(),
                message: "DABADKKK/1234567890".to_string(),
            },
            Record {
                tag: "28C".to_string(),
                message: "00014/001".to_string(),
            },
            Record {
                tag: "60F".to_string(),
                message: "C091019DKK3859701,48".to_string(),
            },
            Record {
                tag: "86".to_string(),
                message: "For your inform. IBAN no.: DK5030001234567890\n\
                          DABADKKK                                                 \n\
                          1234567890\n\
                          DANSKE BANK                        HOLMENS KANAL 2-12\
                          "
                .to_string(),
            },
            Record {
                tag: "61".to_string(),
                message: "0910201020DK5312,50NMSCDBT.teste kunden".to_string(),
            },
            Record {
                tag: "86".to_string(),
                message: "F.M.T.\n\
                          V/TESTE KUNDEN\n\
                          HOLMENS KANAL 2-12\n\
                          1192  KOBENHAVN H\
                          "
                .to_string(),
            },
            Record {
                tag: "61".to_string(),
                message: "0910201020DK3009,51NMSCDBT.Pet Van Park".to_string(),
            },
            Record {
                tag: "86".to_string(),
                message: "DBTS 1111272333/Bnf. PET VAN PARK AMSTERDAM/Bnf.acc. NL47ABNAXXXX\n\
                          558756/Our fee DKK 40,00/Foreign fee DKK 200,00"
                    .to_string(),
            },
            Record {
                tag: "62F".to_string(),
                message: "C091020DKK3851379,47".to_string(),
            },
            Record {
                tag: "64".to_string(),
                message: "C091020DKK3851379,47".to_string(),
            },
        ];

        let result = parse_mt940(test_data).unwrap();
        assert_eq!(expected, result);
    }
}
