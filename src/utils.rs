use rust_decimal::Decimal;
use chrono::prelude::*;
use pest::Parser;

use errors::AmountParseError;
use MT940Parser;
use Rule;

/// Create a `Decimal` from a MT940 amount.
///
/// MT940 amounts always have a comma as a decimal separator.
/// However, they might not always have digits behind the comma.
pub fn decimal_from_mt940_amount(s: &str) -> Result<Decimal, AmountParseError> {
    // Split at decimal separator.
    let split_decimal_str: Vec<&str> = s.split(",").collect();
    if split_decimal_str.len() == 1 {
        return Err(AmountParseError::NoComma);
    } else if split_decimal_str.len() > 2 {
        return Err(AmountParseError::TooManyCommas);
    }
    let (int_part, frac_part) = (split_decimal_str[0], split_decimal_str[1]);
    let whole_number: i64 = format!("{}{}", int_part, frac_part)
        .parse()
        .map_err(|x| AmountParseError::IntParseError(x))?;
    Ok(Decimal::new(whole_number, frac_part.len() as u32))
}

/// Create a `NaiveDate` from a MT940 date.
///
/// MT940 has a weird date format in the form of YYMMDD. Since it has a shortened year, the
/// assumption is made that all statement are in the year 20XX.
pub fn date_from_mt940_date(s: &str) -> Result<NaiveDate, pest::error::Error<Rule>> {
    let parsed_date = MT940Parser::parse(Rule::date, &s)?.next().unwrap().into_inner();
    let mut year = None;
    let mut month = None;
    let mut day = None;
    for p in parsed_date {
        match p.as_rule() {
            // Here I'm making an assumption that will only work for
            // a limited but fairly long time: That all years that we
            // see are at least the year 2000 and upwards. The
            // problem is sadly that banks didn't make the field be the
            // full year number but only a 2-digit number!
            // How stupid.
            Rule::year => year = Some(format!("20{}", p.as_str())),
            Rule::month => month = Some(p.as_str()),
            Rule::day => day = Some(p.as_str()),
            _ => unreachable!(),
        }
    }
    Ok(NaiveDate::from_ymd(
        year.unwrap().parse().unwrap(),
        month.unwrap().parse().unwrap(),
        day.unwrap().parse().unwrap(),
    ))
}
