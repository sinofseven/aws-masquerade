# aws-masquerade

A lightweight Rust CLI tool to manage AWS AssumeRole operations with support for TOTP-based MFA, flexible credential output formats, and version 1 configuration in TOML.

## Features

- **Flexible Source/Target Configuration** — Separate AWS account credentials (source) from target roles (target), allowing multiple targets to share the same source credentials
- **TOTP-Based MFA** — Automatically generate TOTP codes from MFA secrets, or provide them interactively
- **Multiple Credential Output Formats** — Export credentials as JSON, Bash, Fish, PowerShell scripts, or directly to `~/.aws/credentials`
- **Version 1 Configuration** — Modern TOML-based configuration replacing legacy JSON format
- **Automatic Migration** — Seamless upgrade path from v0 (JSON) to v1 (TOML) configuration
- **Cross-Platform** — Pre-built binaries for Linux (x86_64, ARM64, ARM), macOS (aarch64), and Windows

## Installation

### From GitHub Releases

Download the latest pre-built binary for your platform from [GitHub Releases](https://github.com/sinofseven/aws-masquerade/releases).

Supported platforms:
- Linux x86_64, ARM64, ARM (glibc/musl)
- macOS aarch64
- Windows x86_64

### Build from Source

Requirements: Rust 1.95.0+

```bash
cargo install --path .
```

This installs the binary to `~/.cargo/bin/aws-masquerade`.

## Quick Start

### 1. Create Configuration File

Create `~/.config/aws-masquerade/config.toml`:

```toml
[core]
version = "1"
save_totp_counter_history = false

# Define your AWS credential source
[[source]]
name = "my-account"
profile = "default"          # AWS profile name
region = "ap-northeast-1"
mfa_arn = "arn:aws:iam::123456789012:mfa/username"
mfa_secret = "JBSWY3DPEBLW64TMMQ======"  # Optional: Base32-encoded MFA secret for TOTP

# Define a target role to assume
[[target]]
name = "dev-role"
source = "my-account"
role_arn = "arn:aws:iam::987654321098:role/DevRole"
credential_output = "bash"
```

### 2. Verify Configuration

```bash
aws-masquerade configure validate
```

### 3. Assume a Role

```bash
# Get credentials in Bash format (ready for eval)
aws-masquerade assume dev-role

# Load credentials into your shell
eval $(aws-masquerade assume dev-role)

# Or specify output format explicitly
aws-masquerade assume dev-role -c json
```

## Configuration File Format

Configuration file location: `~/.config/aws-masquerade/config.toml`

### Core Section

```toml
[core]
version = "1"
save_totp_counter_history = false  # Track used TOTP counters to prevent reuse
```

### Source Section

Define where to get AWS credentials for assuming roles:

```toml
[[source]]
name = "production"              # Required: identifier for this source
profile = "prod-profile"         # AWS profile name (or use credentials below)
region = "us-east-1"             # AWS region (default: us-east-1)
mfa_arn = "arn:aws:iam::..."     # MFA device ARN (optional)
mfa_secret = "BASE32..."         # TOTP secret in Base32 encoding (optional)
note = "Production AWS account"  # Free-form notes (optional)

# Alternative: use static credentials instead of profile
# aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"
# aws_secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Unique identifier for this source |
| `profile` | String | No | AWS profile name (from ~/.aws/config). If omitted, uses `name` as profile. |
| `region` | String | No | AWS region (default: us-east-1) |
| `mfa_arn` | String | No | ARN of MFA device for multi-factor authentication |
| `mfa_secret` | String | No | Base32-encoded TOTP secret for automatic MFA code generation |
| `aws_access_key_id` | String | No | AWS Access Key ID (use instead of profile) |
| `aws_secret_access_key` | String | No | AWS Secret Access Key (required with access_key_id) |
| `note` | String | No | Free-form notes about this source |

### Target Section

Define roles to assume using source credentials:

```toml
[[target]]
name = "app-deploy"              # Required: identifier for this target
source = "production"            # Required: name of source to use
role_arn = "arn:aws:iam::..."    # Required: role to assume
credential_output = "bash"       # Required: output format
duration_seconds = 3600          # Session duration in seconds (900-43200)
region = "ap-northeast-1"        # Region after assuming role (optional)
cli_output = "json"              # AWS CLI output format: json|yaml|text|table (optional)
note = "Deployment role"         # Free-form notes (optional)
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Unique identifier for this target |
| `source` | String | Yes | Name of the source to use for credentials |
| `role_arn` | String | Yes | ARN of the role to assume |
| `credential_output` | String | Yes | Output format: json, bash, fish, PowerShell, SharedCredentials |
| `duration_seconds` | Integer | No | Session duration in seconds (min: 900, max: 43200) |
| `region` | String | No | AWS region for the assumed role |
| `cli_output` | String | No | AWS CLI output format: json, yaml, yaml-stream, text, or table |
| `note` | String | No | Free-form notes |

## Command Reference

### configure — Configuration Management

```bash
# Show the current configuration file path
aws-masquerade configure path

# Validate the configuration for correctness
aws-masquerade configure validate

# Migrate from v0 (JSON) to v1 (TOML) format
aws-masquerade configure migrate
```

### source — View Credential Sources

```bash
# List all defined sources (JSON output)
aws-masquerade source list

# Show details of a specific source
aws-masquerade source show <SOURCE_NAME>
```

### target — View Target Roles

```bash
# List all defined targets (JSON output)
aws-masquerade target list

# Show details of a specific target
aws-masquerade target show <TARGET_NAME>
```

### assume — Execute AssumeRole

```bash
# Assume a role with default output format
aws-masquerade assume <TARGET_NAME>

# Specify output format
aws-masquerade assume <TARGET_NAME> -c <FORMAT>
aws-masquerade assume <TARGET_NAME> --credential-output <FORMAT>
```

Supported formats:
- `json` or `j` — JSON object with credentials
- `bash` or `b` — Bash export commands
- `fish` or `f` — Fish shell set commands
- `PowerShell` or `p` — PowerShell variable assignments
- `SharedCredentials` or `s` — Write directly to ~/.aws/credentials

## Shell Integration

### Bash

```bash
# Load credentials into current Bash session
eval $(aws-masquerade assume dev-role)

# Or with custom output format
eval $(aws-masquerade assume dev-role -c bash)
```

### Fish

```fish
# Load credentials into current Fish session
aws-masquerade assume dev-role -c fish | source

# Or using default format if configured as Fish
aws-masquerade assume dev-role | source
```

### PowerShell

```powershell
# Load credentials into current PowerShell session
aws-masquerade assume dev-role -c PowerShell | Invoke-Expression
```

## MFA Setup

### Using TOTP for Automatic MFA

1. **Enable MFA in AWS IAM Console**
   - Create a virtual MFA device
   - Copy the secret key (base32-encoded string)

2. **Configure in aws-masquerade**
   ```toml
   [[source]]
   name = "my-account"
   profile = "default"
   mfa_arn = "arn:aws:iam::123456789012:mfa/username"
   mfa_secret = "JBSWY3DPEBLW64TMMQ======"  # Your base32-encoded secret
   ```

3. **AssumeRole will now auto-generate TOTP codes**
   ```bash
   aws-masquerade assume my-target
   # Automatically generates TOTP code from mfa_secret
   ```

### Interactive MFA (Without Secret)

If you don't have `mfa_secret` configured:

```bash
aws-masquerade assume my-target
# Prompts for MFA token
MFA TOKEN: 123456
```

### TOTP Counter History

When `save_totp_counter_history = true` (default), aws-masquerade tracks used TOTP counters in `~/.config/aws-masquerade/.totp_count_history.json` to prevent token reuse. This is necessary because AWS rejects duplicate TOTP codes.

## Migration from v0 to v1

If you have an existing v0 configuration in JSON format (`~/.config/aws-masquerade/config.json`), migrate to v1 TOML format:

```bash
aws-masquerade configure migrate
```

This automatically converts your configuration and creates `~/.config/aws-masquerade/config.toml`.

### Format Comparison

**v0 (JSON):**
```json
{
  "accounts": {
    "dev": {
      "sourceProfile": "default",
      "roleArn": "arn:aws:iam::987654321098:role/DevRole",
      "credentialOutput": "bash"
    }
  }
}
```

**v1 (TOML):**
```toml
[[source]]
name = "default"
profile = "default"

[[target]]
name = "dev"
source = "default"
role_arn = "arn:aws:iam::987654321098:role/DevRole"
credential_output = "bash"
```

### Key Changes

- **Structure**: Single `account` split into separate `source` (credentials) and `target` (roles)
- **Format**: JSON → TOML
- **Reusability**: Multiple targets can share the same source credentials
- **Flexibility**: Different authentication methods (profiles vs static keys) per source

## FAQ & Troubleshooting

### Q: How do I use static AWS credentials instead of a profile?

A: Replace the `profile` field with static credentials:

```toml
[[source]]
name = "static-creds"
aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"
aws_secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
```

### Q: Can multiple targets share the same source?

A: Yes, this is a key feature of v1. Define one source and reference it from multiple targets:

```toml
[[source]]
name = "prod-account"
profile = "prod"

[[target]]
name = "prod-app-role"
source = "prod-account"
role_arn = "arn:aws:iam::123456789012:role/AppRole"

[[target]]
name = "prod-admin-role"
source = "prod-account"
role_arn = "arn:aws:iam::123456789012:role/AdminRole"
```

### Q: What's the difference between `credential_output` and `cli_output`?

A: 
- `credential_output` — How to output AWS credentials (bash, fish, PowerShell, etc.)
- `cli_output` — Format for AWS CLI commands (json, yaml, text, table)

### Q: Why does my configuration validation fail?

A: Common issues:
- `source` name in a target doesn't exist
- Duplicate names in sources or targets
- `mfa_secret` without `mfa_arn`
- `aws_secret_access_key` without `aws_access_key_id`

Run `aws-masquerade configure validate` for detailed error messages.

### Q: How do I prevent MFA token reuse errors?

A: Ensure `save_totp_counter_history = true` in the `[core]` section. This prevents using the same TOTP code within the same 30-second window.

## Development

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Type Check & Lint

```bash
cargo check    # Type checking only (fast)
cargo clippy   # Lint warnings
```

### Test

Tests are not yet implemented. Functionality is verified by running actual subcommands:

```bash
cargo run -- configure validate
cargo run -- assume my-target
```

### Release

Releases are automated via GitHub Actions:

1. Push a version tag: `git tag v0.4.0 && git push origin v0.4.0`
2. GitHub Actions automatically builds for all platforms
3. Artifacts are uploaded to [GitHub Releases](https://github.com/sinofseven/aws-masquerade/releases)

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

Third-party licenses are included in `THIRD_PARTY_LICENSES.html` in release packages.

---

For detailed architecture and design decisions, see [CLAUDE.md](./CLAUDE.md).

Author: [sinofseven](https://github.com/sinofseven)  
Repository: https://github.com/sinofseven/aws-masquerade
