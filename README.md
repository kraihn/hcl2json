# hcl2json

A fast and reliable CLI tool to convert HCL (HashiCorp Configuration Language) files to JSON format.

## Features

- **Convert HCL to JSON** with proper type preservation
- **Property extraction** with dot notation support (`database.engine`)
- **Multi-file processing** with automatic merging
- **Deep merge** for combining nested objects across files
- **Validation mode** to check HCL syntax without conversion
- **Flexible formatting** with pretty printing and custom indentation
- **Glob pattern support** for batch processing
- **Single quotes output** with proper escaping
- **Comprehensive error messages** with helpful suggestions

## Installation

```bash
cargo install --path .
```

## Usage

### Basic Conversion

```bash
# Convert HCL file to JSON
hcl2json --file config.tfvars

# Read from stdin
cat config.tfvars | hcl2json

# Pretty format with indentation
hcl2json --pretty --file config.tfvars

# Custom indentation (4 spaces)
hcl2json --pretty --indent 4 --file config.tfvars
```

### Property Extraction

```bash
# Extract specific property
hcl2json --file config.tfvars --property tags

# Extract nested property
hcl2json --file config.tfvars --property database.engine
```

### Multi-File Processing

```bash
# Merge multiple files (shallow merge by default)
hcl2json --file config1.tfvars --file config2.tfvars

# Deep merge (combines nested objects)
hcl2json --deep-merge --file config1.tfvars --file config2.tfvars

# Use glob patterns
hcl2json --file "configs/*.tfvars"
```

### Validation

```bash
# Validate HCL syntax without conversion
hcl2json --validate --file config.tfvars

# Validate multiple files
hcl2json --validate --file "*.tfvars"
```

### Output Options

```bash
# Save to file
hcl2json --file config.tfvars --output result.json

# Use single quotes instead of double quotes
hcl2json --single-quotes --file config.tfvars
```

## Examples

### Input HCL (`terraform.tfvars`)
```hcl
region = "us-west-2"
instance_type = "t3.micro"

database = {
  engine  = "mysql"
  version = "8.0"
  port    = 3306
}

tags = {
  Environment = "production"
  Project     = "web-app"
}
```

### Basic Conversion
```bash
$ hcl2json --file terraform.tfvars
{"database":{"engine":"mysql","port":3306,"version":"8.0"},"instance_type":"t3.micro","region":"us-west-2","tags":{"Environment":"production","Project":"web-app"}}
```

### Pretty Format
```bash
$ hcl2json --pretty --file terraform.tfvars
{
  "database": {
    "engine": "mysql",
    "port": 3306,
    "version": "8.0"
  },
  "instance_type": "t3.micro",
  "region": "us-west-2",
  "tags": {
    "Environment": "production",
    "Project": "web-app"
  }
}
```

### Property Extraction
```bash
$ hcl2json --file terraform.tfvars --property database.engine
"mysql"
```

### Validation
```bash
$ hcl2json --validate --file terraform.tfvars
VALID: terraform.tfvars
```

## Merge Behavior

### Shallow Merge (Default)
When multiple files have the same top-level key, the last file wins:

```bash
# config1.tfvars: tags = { Team = "backend" }
# config2.tfvars: tags = { Environment = "prod" }
# Result: tags = { Environment = "prod" }  # config1's tags lost
```

### Deep Merge
Recursively combines nested objects:

```bash
# Same files with --deep-merge
# Result: tags = { Team = "backend", Environment = "prod" }  # both preserved
```

## Error Handling

The tool provides helpful error messages:

```bash
$ hcl2json --file config.tfvars --property nonexistent
Error: Property 'nonexistent' not found at 'nonexistent' (available properties: database, instance_type, region, tags)
```

## Command Line Options

```
Options:
      --pretty               Pretty format JSON with newlines and indentation
      --indent <INDENT>      Number of spaces for indentation (default: 2)
      --validate             Validate HCL syntax without conversion
      --single-quotes        Use single quotes instead of double quotes
  -o, --output <OUTPUT>      Output file (stdout if not specified)
  -f, --file <FILE>          HCL file(s) to convert (supports glob patterns)
      --deep-merge           Use deep merge when multiple files provided
  -p, --property <PROPERTY>  Property within HCL to extract (optional)
      --version              Print version
  -h, --help                 Print help
```

## Development

### Running Tests
```bash
cargo test
```

### Code Coverage
```bash
cargo tarpaulin
```

### Building
```bash
cargo build --release
```

## License

MIT License - see LICENSE file for details.
