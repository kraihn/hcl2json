use anyhow::Result;
use clap::Parser;
use hcl2json::{process_hcl, Config};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hcl2json")]
#[command(about = "Convert HCL files to JSON")]
struct Args {
    /// Pretty format JSON with newlines and indentation
    #[arg(long)]
    pretty: bool,

    /// Number of spaces for indentation
    #[arg(long, default_value = "2")]
    indent: usize,

    /// Validate HCL syntax without conversion
    #[arg(long)]
    validate: bool,

    /// Use single quotes instead of double quotes
    #[arg(long)]
    single_quotes: bool,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// HCL file(s) to convert (supports glob patterns, reads from stdin if not provided)
    #[arg(short, long, value_name = "FILE")]
    file: Vec<String>,

    /// Use deep merge instead of shallow merge when multiple files provided
    #[arg(long)]
    deep_merge: bool,

    /// Property within HCL to extract (optional)
    #[arg(short, long)]
    property: Option<String>,

    /// Print version
    #[arg(long)]
    version: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!("hcl2json {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let input = if args.file.is_empty() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Some(buffer)
    } else {
        None
    };

    let config = Config {
        pretty: args.pretty,
        indent: args.indent,
        validate: args.validate,
        single_quotes: args.single_quotes,
        files: args.file,
        deep_merge: args.deep_merge,
        property: args.property,
    };

    let result = process_hcl(config, input)?;

    if let Some(output_path) = args.output {
        fs::write(output_path, result)?;
    } else {
        println!("{}", result);
    }

    Ok(())
}
