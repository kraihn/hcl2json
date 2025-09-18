use anyhow::{bail, Context, Result};
use clap::Parser;
use glob::glob;
use hcl::Value;
use serde_json::Value as JsonValue;
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
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let files = get_input_files(&args.file)?;
    let contents = read_files(&files)?;

    if args.validate {
        return validate_hcl_files(&contents);
    }

    let json_value = if contents.len() > 1 {
        merge_hcl_contents(&contents, args.deep_merge)?
    } else if contents.len() == 1 {
        parse_hcl_content(&contents[0])?
    } else {
        bail!("No input provided");
    };

    let final_value = if let Some(property) = &args.property {
        extract_property(&json_value, property)?
    } else {
        json_value
    };

    let json_string = format_json(&final_value, &args)?;

    let output = if args.single_quotes {
        // Replace JSON structure quotes but preserve escaped quotes in values
        json_string
            .replace("\\\"", "ESCAPED_QUOTE_PLACEHOLDER")
            .replace('"', "'")
            .replace("ESCAPED_QUOTE_PLACEHOLDER", "\\\"")
    } else {
        json_string
    };

    match args.output {
        Some(path) => fs::write(path, output)?,
        None => println!("{}", output),
    }

    Ok(())
}

fn get_input_files(file_patterns: &[String]) -> Result<Vec<PathBuf>> {
    if file_patterns.is_empty() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    for pattern in file_patterns {
        let matches =
            glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;

        for entry in matches {
            let path =
                entry.with_context(|| format!("Error reading glob match for: {}", pattern))?;
            files.push(path);
        }
    }

    if files.is_empty() {
        bail!("No files found matching the provided patterns");
    }

    Ok(files)
}

fn read_files(files: &[PathBuf]) -> Result<Vec<(String, String)>> {
    if files.is_empty() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .with_context(|| "Failed to read from stdin")?;
        return Ok(vec![("stdin".to_string(), buffer)]);
    }

    let mut contents = Vec::new();
    for file in files {
        let content = fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?;
        contents.push((file.display().to_string(), content));
    }

    Ok(contents)
}

fn validate_hcl_files(contents: &[(String, String)]) -> Result<()> {
    let mut has_errors = false;

    for (name, content) in contents {
        match hcl::from_str::<Value>(content) {
            Ok(_) => println!("VALID: {}", name),
            Err(e) => {
                println!("INVALID: {} - {}", name, format_hcl_error(&e));
                has_errors = true;
            }
        }
    }

    if has_errors {
        bail!("Validation failed: one or more files contain invalid HCL");
    }

    Ok(())
}

fn parse_hcl_content((name, content): &(String, String)) -> Result<JsonValue> {
    let hcl_value: Value = hcl::from_str(content).with_context(|| {
        format!(
            "Failed to parse HCL in {}: {}",
            name,
            format_hcl_error_simple(content)
        )
    })?;

    hcl_to_json(hcl_value)
}

fn merge_hcl_contents(contents: &[(String, String)], deep: bool) -> Result<JsonValue> {
    let mut merged = serde_json::Map::new();

    for (name, content) in contents {
        let hcl_value: Value =
            hcl::from_str(content).with_context(|| format!("Failed to parse HCL in {}", name))?;

        let json_value = hcl_to_json(hcl_value)?;

        if let JsonValue::Object(obj) = json_value {
            if deep {
                deep_merge_objects(&mut merged, obj);
            } else {
                for (key, value) in obj {
                    merged.insert(key, value);
                }
            }
        } else {
            bail!("Cannot merge non-object HCL content from: {}", name);
        }
    }

    Ok(JsonValue::Object(merged))
}

fn deep_merge_objects(
    target: &mut serde_json::Map<String, JsonValue>,
    source: serde_json::Map<String, JsonValue>,
) {
    for (key, value) in source {
        match (target.get_mut(&key), &value) {
            (Some(JsonValue::Object(target_obj)), JsonValue::Object(source_obj)) => {
                deep_merge_objects(target_obj, source_obj.clone());
            }
            _ => {
                target.insert(key, value);
            }
        }
    }
}

fn format_json(value: &JsonValue, args: &Args) -> Result<String> {
    if args.pretty {
        let pretty = serde_json::to_string_pretty(value)?;
        if args.indent != 2 {
            // Replace default 2-space indentation with custom
            let lines: Vec<String> = pretty
                .lines()
                .map(|line| {
                    let leading_spaces = line.len() - line.trim_start().len();
                    let indent_level = leading_spaces / 2;
                    let new_indent = " ".repeat(indent_level * args.indent);
                    format!("{}{}", new_indent, line.trim_start())
                })
                .collect();
            Ok(lines.join("\n"))
        } else {
            Ok(pretty)
        }
    } else {
        Ok(serde_json::to_string(value)?)
    }
}

fn format_hcl_error(error: &hcl::Error) -> String {
    match error {
        hcl::Error::Parse(parse_err) => {
            format!("Parse error: {}", parse_err)
        }
        _ => format!("{}", error),
    }
}

fn format_hcl_error_simple(content: &str) -> String {
    let lines = content.lines().count();
    format!("syntax error (file has {} lines)", lines)
}

fn hcl_to_json(value: Value) -> Result<JsonValue> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(b)),
        Value::Number(n) => Ok(serde_json::to_value(n)?),
        Value::String(s) => Ok(JsonValue::String(s)),
        Value::Array(arr) => {
            let json_arr: Result<Vec<JsonValue>> = arr.into_iter().map(hcl_to_json).collect();
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

    for (i, part) in parts.iter().enumerate() {
        match current.get(part) {
            Some(value) => current = value,
            None => {
                let available_keys = match current {
                    JsonValue::Object(obj) => {
                        let keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                        if keys.is_empty() {
                            "no properties available".to_string()
                        } else {
                            format!("available properties: {}", keys.join(", "))
                        }
                    }
                    _ => "not an object".to_string(),
                };

                let path_so_far = parts[..=i].join(".");
                bail!(
                    "Property '{}' not found at '{}' ({})",
                    property,
                    path_so_far,
                    available_keys
                );
            }
        }
    }

    Ok(current.clone())
}
