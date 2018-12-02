#![feature(test)]

extern crate test;

use mt940::parse_mt940;
use test::Bencher;

static LONG_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/danskebank/MT940_DK_Example.sta");
static SHORT_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/danskebank/MT940_FI_Example.sta");

#[bench]
fn bench_long_statement(b: &mut Bencher) {
    b.iter(|| parse_mt940(&LONG_STATEMENT).unwrap());
}

#[bench]
fn bench_short_statement(b: &mut Bencher) {
    b.iter(|| parse_mt940(&SHORT_STATEMENT).unwrap());
}
