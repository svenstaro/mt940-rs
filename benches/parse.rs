#![feature(test)]

extern crate test;

use mt940::parse_mt940;
use mt940::sanitizers::sanitize;
use test::Bencher;

static LONGER_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/betterplace/sepa_mt9401.sta");
static LONG_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/danskebank/MT940_DK_Example.sta");
static SHORT_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/danskebank/MT940_FI_Example.sta");

#[bench]
fn bench_longer_statement_with_sanitize(b: &mut Bencher) {
    b.iter(||
        parse_mt940(&sanitize(&LONGER_STATEMENT)).unwrap());
}

#[bench]
fn bench_longer_statement_presanitize(b: &mut Bencher) {
    let sanitized = sanitize(&LONGER_STATEMENT);
    b.iter(||
        parse_mt940(&sanitized).unwrap());
}

#[bench]
fn bench_long_statement(b: &mut Bencher) {
    b.iter(|| parse_mt940(&LONG_STATEMENT).unwrap());
}

#[bench]
fn bench_short_statement(b: &mut Bencher) {
    b.iter(|| parse_mt940(&SHORT_STATEMENT).unwrap());
}
