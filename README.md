# mt940-rs

[![Build Status](https://travis-ci.com/svenstaro/mt940-rs.svg?branch=master)](https://travis-ci.org/svenstaro/mt940-rs)
[![Docs Status](https://docs.rs/mt940/badge.svg)](https://docs.rs/mt940)
[![codecov](https://codecov.io/gh/svenstaro/mt940-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/svenstaro/mt940-rs)
[![Crates.io](https://img.shields.io/crates/v/mt940-rs.svg)](https://crates.io/crates/mt940-rs)
[![dependency status](https://deps.rs/repo/github/svenstaro/mt940-rs/status.svg)](https://deps.rs/repo/github/svenstaro/mt940-rs)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/mt940-rs/blob/master/LICENSE)

**A strict MT940 bank statement parser in Rust.**

## Features

- Parse MT940 bank statements.
- Strict and well-researched.
- Super simple API and nice Rusty structs.
- Small commandline utility that allows for quick and easy conversion of MT940 statements to JSON.

## Planned features

- MT941 support
- MT942 support

## Library usage example

```rust
extern crate mt940;
use mt940::parse_mt940;

fn main() {
    let input = "\
        :20:3996-11-11111111\r\n\
        :25:DABADKKK/111111-11111111\r\n\
        :28C:00001/001\r\n\
        :60F:C090924EUR54484,04\r\n\
        :61:0909250925DR583,92NMSC1110030403010139//1234\r\n\
        :86:11100304030101391234\r\n\
        Beneficiary name\r\n\
        Something else\r\n\
        :61:0910010930DR62,60NCHGcustomer id//bank id\r\n\
        :86:Fees according to advice\r\n\
        :62F:C090930EUR53126,94\r\n\
        :64:C090930EUR53189,31\r\n\
        \r\n";

    let input_parsed = parse_mt940(input).unwrap();
    assert_eq!(input_parsed[0].transaction_ref_no, "3996-11-11111111");
}
```

## CLI usage example

    cargo run --bin sta2json tests/data/mt940/full/danskebank/MT940_DK_Example.sta
    
## Documentation

Documentation is [here](https://docs.rs/mt940).

## Caveats

Some banks bank use weird derivates of MT940 that are do not strictly follow the specification.
In that case, I recommend you do some pre-processing of those statements.

## Resources and acknowledgements

Referencing proper docs is important because because banks seem to be somewhat lenient about their strictness in implementing MT940. Below I assembled a list of resources that I reference.

### Other projects

- Lots of test data copied from https://github.com/WoLpH/mt940

### iotafinance.com

Amazing interactive docs.

- http://www.iotafinance.com/en/SWIFT-ISO15022-Message-types-in-category-9.html

### DanskeBank

They provide tons of good docs.

- https://danskebank.com/en-uk/ci/Products-Services/Transaction-Services/Online-Services/Integration-Services/Documents/Formats/FormatDescriptionSWIFT_MT940/MT940.pdf
- https://danskebank.com/en-uk/ci/Products-Services/Transaction-Services/Online-Services/Integration-Services/Documents/Formats/FormatDescriptionSWIFT_MT942/MT942.pdf
- https://danskebank.com/en-uk/ci/Products-Services/Transaction-Services/Online-Services/Integration-Services/Formats/Pages/SWIFT-formats.aspx
- https://www-2.danskebank.com/Link/ExtendedMT940/$file/Extended_MT940.pdf

### Bank Austria

- https://www.bankaustria.at/files/MBS_MT940_V5107.pdf

### Deutsche Bank

- https://deutschebank.nl/nl/docs/MT94042_EN.pdf

### ABN AMRO

- https://www.abnamro.nl/nl/images/Generiek/PDFs/020_Zakelijk/03_OfficeNet/Formatenboek_MT94_(engels).pdf
- https://www.abnamro.nl/en/images/Generiek/PDFs/020_Zakelijk/04_Migratie/Derde_banken_Specificaties_EN.pdf

### Westpac Banking

- https://quickstream.westpac.com.au/docs/quickrec/#mt940-file-format-specification

### Societe Generale Srbija
- https://web.archive.org/web/20160725042101/http://www.societegenerale.rs/fileadmin/template/main/pdf/SGS%20MT940.pdf

### Bank Millennium

- https://www.bankmillennium.pl/documents/10184/128700/File_format_description_of_MT940_v20120309_1216901.pdf/d56bc65f-fa84-4d6c-b07c-9ca60381016b


### DZ Bank

- https://www.dzbank.de/content/dam/dzbank_de/de/home/produkte_services/Firmenkunden/PDF-Dokumente/transaction%20banking/elektronicBanking/SEPA-Belegungsregeln_MT940-DK_082016.~644b217ec96b35dfffcaf18dc2df800a.pdf

### Handelsbanken

- https://www.handelsbanken.se/shb/inet/icentsv.nsf/vlookuppics/a_filmformatbeskrivningar_fil_mt940_account_statement_20081212/$file/mt940_account_statement.pdf

### ING Bank

- https://www.ing.nl/media/ING_Format-Description-Structured-Unstructured-MT940-MT942-IBP-v5.2_tcm162-140454.pdf

### Kontopruef

- https://www.kontopruef.de/mt940s.shtml

### Rabo Bank

- https://www.rabobank.com/en/images/Format_Description_MT940%20_2.4_EN.pdf

### SEPA for Corporates

- http://www.sepaforcorporates.com/swift-for-corporates/account-statement-mt940-file-format-overview/
- http://www.sepaforcorporates.com/swift-for-corporates/list-mt940-transaction-type-identification-codes/
- http://www.sepaforcorporates.com/swift-for-corporates/quick-guide-swift-mt101-format/
