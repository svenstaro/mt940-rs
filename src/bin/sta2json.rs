use clap::Parser;
use mt940::parse_mt940;
use mt940::sanitizers::sanitize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Convert mt940 statement to json.
///
/// It will try to sanitize input by default. For instance, it will
/// try to fit all found characters into the allowable SWIFT charset
/// before attempting conversion. You can turn off this behavior by
/// enabling strict mode.
#[derive(Parser)]
#[clap(name = "sta2json", author, about, version)]
pub struct Args {
    /// Enable strict parsing. When this is on, input won't be sanitized.
    #[clap(short, long)]
    pub strict: bool,

    /// Input mt940 statement.
    #[clap()]
    pub statement: PathBuf,

    /// Output file in JSON format.
    #[clap()]
    pub output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut input = fs::read_to_string(args.statement)?;

    // Do some sanitizing if not running in strict mode.
    if !args.strict {
        input = sanitize(&input);
    }

    let parsed = parse_mt940(&input).unwrap_or_else(|e| panic!("{}", e));

    let json = serde_json::to_string_pretty(&parsed)?;

    if let Some(output) = args.output {
        // Write to a file.
        fs::write(output, json)?;
    } else {
        // Write to stdout instead.
        io::stdout().write_all(json.as_bytes())?;
    };

    Ok(())
}
