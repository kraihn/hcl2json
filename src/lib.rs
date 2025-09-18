use anyhow::{bail, Context, Result};
use glob::glob;
use hcl::Value;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::fs;

pub struct Config {
    pub pretty: bool,
    pub indent: usize,
    pub validate: bool,
    pub single_quotes: bool,
    pub files: Vec<String>,
    pub deep_merge: bool,
    pub property: Option<String>,
}

pub fn process_hcl(config: Config, input: Option<String>) -> Result<String> {
    if config.validate {
        return validate_files(&config.files, input);
    }

    let merged_value = if config.files.is_empty() {
        let content = input.context("No input provided")?;
        parse_hcl_content(&content)?
    } else {
        merge_files(&config.files, config.deep_merge)?
    };

    let result_value = if let Some(property) = &config.property {
        extract_property(&merged_value, property)?
    } else {
        merged_value
    };

    format_output(
        &result_value,
        config.pretty,
        config.indent,
        config.single_quotes,
    )
}

fn validate_files(files: &[String], input: Option<String>) -> Result<String> {
    if files.is_empty() {
        if let Some(content) = input {
            parse_hcl_content(&content)?;
            return Ok("VALID: stdin".to_string());
        }
        bail!("No files or input provided for validation");
    }

    let mut results = Vec::new();
    for file_pattern in files {
        for entry in glob(file_pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)?;
            parse_hcl_content(&content)?;
            results.push(format!("VALID: {}", path.display()));
        }
    }
    Ok(results.join("\n"))
}

fn merge_files(files: &[String], deep_merge: bool) -> Result<JsonValue> {
    let mut merged = JsonValue::Object(serde_json::Map::new());

    for file_pattern in files {
        for entry in glob(file_pattern)? {
            let path = entry?;
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            let value = parse_hcl_content(&content)?;

            if deep_merge {
                deep_merge_json(&mut merged, value);
            } else {
                shallow_merge_json(&mut merged, value);
            }
        }
    }

    Ok(merged)
}

fn parse_hcl_content(content: &str) -> Result<JsonValue> {
    let hcl_value: Value = hcl::from_str(content).context("Failed to parse HCL content")?;

    let json_string = serde_json::to_string(&hcl_value).context("Failed to convert HCL to JSON")?;

    serde_json::from_str(&json_string).context("Failed to parse JSON")
}

fn extract_property(value: &JsonValue, property: &str) -> Result<JsonValue> {
    let parts: Vec<&str> = property.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        match current {
            JsonValue::Object(map) => {
                if let Some(next_value) = map.get(*part) {
                    current = next_value;
                } else {
                    let available_keys: Vec<String> = map.keys().cloned().collect();
                    let path = parts[..=i].join(".");
                    bail!(
                        "Property '{}' not found at '{}' (available properties: {})",
                        property,
                        path,
                        available_keys.join(", ")
                    );
                }
            }
            _ => {
                let path = parts[..i].join(".");
                bail!(
                    "Cannot access property '{}' on non-object at path '{}'",
                    parts[i],
                    path
                );
            }
        }
    }

    Ok(current.clone())
}

fn deep_merge_json(target: &mut JsonValue, source: JsonValue) {
    match (target, source) {
        (JsonValue::Object(target_map), JsonValue::Object(source_map)) => {
            for (key, value) in source_map {
                if let Some(existing_value) = target_map.get_mut(&key) {
                    deep_merge_json(existing_value, value);
                } else {
                    target_map.insert(key, value);
                }
            }
        }
        (target, source) => *target = source,
    }
}

fn shallow_merge_json(target: &mut JsonValue, source: JsonValue) {
    if let (JsonValue::Object(target_map), JsonValue::Object(source_map)) = (target, source) {
        for (key, value) in source_map {
            target_map.insert(key, value);
        }
    }
}

fn format_output(
    value: &JsonValue,
    pretty: bool,
    indent: usize,
    single_quotes: bool,
) -> Result<String> {
    let json_string = if pretty {
        let indent_bytes = vec![b' '; indent];
        let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
        let mut buf = Vec::new();
        let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);
        value.serialize(&mut serializer)?;
        String::from_utf8(buf)?
    } else {
        serde_json::to_string(value)?
    };

    if single_quotes {
        Ok(json_string.replace('"', "'"))
    } else {
        Ok(json_string)
    }
}
