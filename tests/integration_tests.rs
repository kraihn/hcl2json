use anyhow::Result;
use hcl2json::{process_hcl, Config};
use serde_json::Value as JsonValue;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_entire_file_conversion() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;
    let json: JsonValue = serde_json::from_str(&result)?;

    assert_eq!(json["region"], "us-west-2");
    assert_eq!(json["instance_type"], "t3.micro");
    assert_eq!(json["enable_monitoring"], true);

    Ok(())
}

#[test]
fn test_single_property_extraction() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: Some("tags".to_string()),
    };

    let result = process_hcl(config, None)?;
    let json: JsonValue = serde_json::from_str(&result)?;

    assert_eq!(json["Environment"], "production");
    assert_eq!(json["Project"], "web-app");

    Ok(())
}

#[test]
fn test_nested_property_extraction() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: Some("database.engine".to_string()),
    };

    let result = process_hcl(config, None)?;
    let json: JsonValue = serde_json::from_str(&result)?;

    assert_eq!(json, "mysql");

    Ok(())
}

#[test]
fn test_pretty_formatting() -> Result<()> {
    let config = Config {
        pretty: true,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;

    assert!(result.contains("{\n"));
    assert!(result.contains("  \"region\""));

    Ok(())
}

#[test]
fn test_single_quotes_output() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: true,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;

    assert!(result.contains("'us-west-2'"));
    assert!(!result.contains("\"us-west-2\""));

    Ok(())
}

#[test]
fn test_validation_mode() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: true,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;

    assert!(result.contains("VALID: test_data/terraform.tfvars"));

    Ok(())
}

#[test]
fn test_stdin_input() -> Result<()> {
    let content = fs::read_to_string("test_data/terraform.tfvars")?;

    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec![],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, Some(content))?;
    let json: JsonValue = serde_json::from_str(&result)?;

    assert_eq!(json["region"], "us-west-2");

    Ok(())
}

#[test]
fn test_multiple_files_shallow_merge() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec![
            "test_data/config1.tfvars".to_string(),
            "test_data/config2.tfvars".to_string(),
        ],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;
    let json: JsonValue = serde_json::from_str(&result)?;

    // In shallow merge, config2 should override config1's tags
    assert_eq!(json["tags"]["Environment"], "staging");
    assert!(
        json["tags"]["Team"].is_null() || !json["tags"].as_object().unwrap().contains_key("Team")
    );

    Ok(())
}

#[test]
fn test_multiple_files_deep_merge() -> Result<()> {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec![
            "test_data/config1.tfvars".to_string(),
            "test_data/config2.tfvars".to_string(),
        ],
        deep_merge: true,
        property: None,
    };

    let result = process_hcl(config, None)?;
    let json: JsonValue = serde_json::from_str(&result)?;

    // In deep merge, both tags should be preserved
    assert_eq!(json["tags"]["Environment"], "staging");
    assert_eq!(json["tags"]["Team"], "backend");

    Ok(())
}

#[test]
fn test_nonexistent_property_error() {
    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: Some("nonexistent".to_string()),
    };

    let result = process_hcl(config, None);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Property 'nonexistent' not found"));
    assert!(error_msg.contains("available properties"));
}

#[test]
fn test_custom_indentation() -> Result<()> {
    let config = Config {
        pretty: true,
        indent: 4,
        validate: false,
        single_quotes: false,
        files: vec!["test_data/terraform.tfvars".to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None)?;

    assert!(result.contains("    \"region\""));

    Ok(())
}

#[test]
fn test_invalid_hcl_syntax() {
    let mut temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "invalid hcl content {").unwrap();

    let config = Config {
        pretty: false,
        indent: 2,
        validate: false,
        single_quotes: false,
        files: vec![temp_file.path().to_string_lossy().to_string()],
        deep_merge: false,
        property: None,
    };

    let result = process_hcl(config, None);
    assert!(result.is_err());
}
