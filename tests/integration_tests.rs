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
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "tags"])
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
        .args(["run", "--", "--file", "test_data/terraform.tfvars", "database.engine"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert_eq!(json_str.trim(), "\"mysql\"");
    
    Ok(())
}

#[test]
fn test_pretty_format() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--pretty", "--file", "test_data/terraform.tfvars", "tags"])
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
        .args(["run", "--", "--single-quotes", "--file", "test_data/terraform.tfvars", "region"])
        .output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    assert_eq!(json_str.trim(), "'us-west-2'");
    
    Ok(())
}

#[test]
fn test_stdin_input() -> Result<()> {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "cat test_data/terraform.tfvars | cargo run -- tags"]);
    let output = cmd.output()?;
    
    assert!(output.status.success());
    
    let json_str = String::from_utf8(output.stdout)?;
    let json: JsonValue = serde_json::from_str(&json_str)?;
    
    assert_eq!(json["Environment"], "production");
    assert_eq!(json["Project"], "web-app");
    
    Ok(())
}
