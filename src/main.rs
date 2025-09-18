use anyhow::{Context, Result};
use clap::Parser;
use hcl::Value;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hcl2json")]
#[command(about = "Convert HCL files to JSON")]
struct Args {
    /// Property within HCL to extract (optional)
    property: Option<String>,
    
    /// Pretty format JSON with newlines and 2-space indentation
    #[arg(long)]
    pretty: bool,
    
    /// Use single quotes instead of double quotes
    #[arg(long)]
    single_quotes: bool,
    
    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// HCL file to convert (reads from stdin if not provided)
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let content = match args.file {
        Some(path) => fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)
                .with_context(|| "Failed to read from stdin")?;
            buffer
        }
    };
    
    let hcl_value: Value = hcl::from_str(&content)
        .with_context(|| "Failed to parse HCL")?;
    
    let json_value = hcl_to_json(hcl_value)?;
    
    let final_value = if let Some(property) = &args.property {
        extract_property(&json_value, property)?
    } else {
        json_value
    };
    
    let json_string = if args.pretty {
        serde_json::to_string_pretty(&final_value)?
    } else {
        serde_json::to_string(&final_value)?
    };
    
    let output = if args.single_quotes {
        json_string.replace('"', "'")
    } else {
        json_string
    };
    
    match args.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{}", output),
    }
    
    Ok(())
}

fn hcl_to_json(value: Value) -> Result<JsonValue> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(b)),
        Value::Number(n) => Ok(serde_json::to_value(n)?),
        Value::String(s) => Ok(JsonValue::String(s)),
        Value::Array(arr) => {
            let json_arr: Result<Vec<JsonValue>> = arr.into_iter()
                .map(hcl_to_json)
                .collect();
            Ok(JsonValue::Array(json_arr?))
        }
        Value::Object(obj) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in obj {
                json_obj.insert(k, hcl_to_json(v)?);
            }
            Ok(JsonValue::Object(json_obj))
        }
    }
}

fn extract_property(json: &JsonValue, property: &str) -> Result<JsonValue> {
    let parts: Vec<&str> = property.split('.').collect();
    let mut current = json;
    
    for part in parts {
        current = current.get(part)
            .with_context(|| format!("Property '{}' not found", property))?;
    }
    
    Ok(current.clone())
}
