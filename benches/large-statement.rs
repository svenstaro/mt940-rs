#![feature(test)]

extern crate mt940;
extern crate test;

use mt940::parse_mt940;
use test::Bencher;

static BENCH_STATEMENT: &str =
    include_str!("../tests/data/mt940/full/danskebank/MT940_DK_Example.sta");
#[bench]
fn bench_long_statement(b: &mut Bencher) {
    b.iter(|| parse_mt940(&BENCH_STATEMENT).unwrap());
}
