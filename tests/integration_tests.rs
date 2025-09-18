use std::process::Command;
use tempfile::NamedTempFile;
use std::fs;
use serde_json::Value as JsonValue;
use anyhow::Result;

#[test]
fn test_entire_file_conversion() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--file", "test_data/terraform.tfvars"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    assert_eq!(json["region"], "us-west-2");
    assert_eq!(json["instance_type"], "t3.micro");
    assert_eq!(json["enable_monitoring"], true);
    
    Ok(())
}

#[test]
fn test_single_property_extraction() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "--property", "tags"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    assert_eq!(json["Environment"], "production");
    assert_eq!(json["Project"], "web-app");
    
    Ok(())
}

#[test]
fn test_nested_property_extraction() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "--property", "database.engine"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert_eq!(json_str.trim(), "\"mysql\"");
    
    Ok(())
}

#[test]
fn test_pretty_format() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--pretty", "--file", "test_data/terraform.tfvars", "--property", "tags"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert!(json_str.contains("  \"Environment\": \"production\""));
    
    Ok(())
}

#[test]
fn test_output_file() -> Result<()> {
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();
    
    let output = Command::new("cargo")
        .args(["run", "--", "-o", temp_path.to_str().unwrap(), "--file", "test_data/terraform.tfvars"])
        .output()?;
    
    assert!(output.status.success());
    
    let content = fs::read_to_string(temp_path)?;
    let json: JsonValue = serde_json::from_str(&content)?;
    assert_eq!(json["region"], "us-west-2");
    
    Ok(())
}

#[test]
fn test_single_quotes() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--single-quotes", "--file", "test_data/terraform.tfvars", "--property", "region"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert_eq!(json_str.trim(), "'us-west-2'");
    
    Ok(())
}

#[test]
fn test_stdin_input() -> Result<()> {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "cat test_data/terraform.tfvars | cargo run -- --property tags"]);
    let output = cmd.output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    assert_eq!(json["Environment"], "production");
    assert_eq!(json["Project"], "web-app");
    
    Ok(())
}

#[test]
fn test_validation_mode_valid() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--validate", "--file", "test_data/terraform.tfvars"])
        .output()?;
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("VALID: test_data/terraform.tfvars"));
    
    Ok(())
}

#[test]
fn test_validation_mode_invalid() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--validate", "--file", "test_data/malformed.hcl"])
        .output()?;
    
    assert!(!output.status.success());
    
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("INVALID: test_data/malformed.hcl"));
    
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Validation failed: one or more files contain invalid HCL"));
    
    Ok(())
}


#[test]
fn test_custom_indentation() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--pretty", "--indent", "4", "--file", "test_data/terraform.tfvars", "--property", "tags"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert!(json_str.contains("    \"Environment\": \"production\""));
    
    Ok(())
}

#[test]
fn test_merge_multiple_files() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "--file", "test_data/network.tfvars"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    // Should have properties from both files
    assert_eq!(json["region"], "us-west-2");
    assert_eq!(json["vpc_cidr"], "10.0.0.0/16");
    
    Ok(())
}

#[test]
fn test_property_not_found_error() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "--property", "nonexistent"])
        .output()?;
    
    assert!(!output.status.success());
    
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Property 'nonexistent' not found"));
    assert!(stderr.contains("available properties:"));
    
    Ok(())
}
#[test]
fn test_deep_merge_multiple_files() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--deep-merge", "--file", "test_data/config1.tfvars", "--file", "test_data/config2.tfvars"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    // Should have properties from both files at top level
    assert_eq!(json["app_settings"]["timeout"], 30);
    assert_eq!(json["network_settings"]["vpc_cidr"], "10.0.0.0/16");
    
    // shared_config should contain nested objects from both files
    assert_eq!(json["shared_config"]["database"]["engine"], "mysql");  // from config1
    assert_eq!(json["shared_config"]["logging"]["level"], "info");     // from config1
    assert_eq!(json["shared_config"]["cache"]["type"], "redis");       // from config2
    assert_eq!(json["shared_config"]["monitoring"]["enabled"], true);  // from config2
    
    Ok(())
}
#[test]
fn test_single_quotes_with_embedded_quotes() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--single-quotes", "--file", "test_data/quotes.tfvars", "--property", "message"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert_eq!(json_str.trim(), "'He said \\\"Hello World\\\" to everyone'");
    
    Ok(())
}
