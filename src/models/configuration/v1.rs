use crate::base::Validation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, clap::ValueEnum)]
pub enum CredentialOutputTarget {
    #[serde(rename = "json")]
    #[value(name = "json", alias = "j", alias = "Json")]
    Json,
    #[serde(rename = "bash")]
    #[value(name = "bash", alias = "b", alias = "Bash")]
    Bash,
    #[serde(rename = "fish")]
    #[value(name = "fish", alias = "f", alias = "Fish")]
    Fish,
    #[value(name = "PowerShell", alias = "p")]
    PowerShell,
    #[value(name = "SharedCredentials", alias = "s")]
    SharedCredentials,
}

impl CredentialOutputTarget {
    pub fn new(text: &str) -> Result<CredentialOutputTarget, String> {
        match text.to_lowercase().as_str() {
            "json" => Ok(CredentialOutputTarget::Json),
            "j" => Ok(CredentialOutputTarget::Json),
            "bash" => Ok(CredentialOutputTarget::Bash),
            "b" => Ok(CredentialOutputTarget::Bash),
            "fish" => Ok(CredentialOutputTarget::Fish),
            "f" => Ok(CredentialOutputTarget::Fish),
            "powershell" => Ok(CredentialOutputTarget::PowerShell),
            "p" => Ok(CredentialOutputTarget::PowerShell),
            "sharedcredentials" => Ok(CredentialOutputTarget::SharedCredentials),
            "s" => Ok(CredentialOutputTarget::SharedCredentials),
            _ => Err(format!("'{}' is not valid credential output.", text)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CliOutputTarget {
    Json,
    Yaml,
    YamlStream,
    Text,
    Table,
}

impl std::fmt::Display for CliOutputTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let result = match self {
            CliOutputTarget::Json => "json",
            CliOutputTarget::Yaml => "yaml",
            CliOutputTarget::YamlStream => "yaml-stream",
            CliOutputTarget::Text => "text",
            CliOutputTarget::Table => "table",
        };
        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Source {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_secret_access_key: Option<String>,
}

impl Validation for Source {
    fn validate(&self) -> Result<(), String> {
        if (self.aws_access_key_id.is_some() && self.aws_secret_access_key.is_none())
            || (self.aws_access_key_id.is_none() && self.aws_secret_access_key.is_some())
        {
            return Err(
                "Both aws_access_key_id and aws_secret_access_key are required".to_string(),
            );
        }

        if self.mfa_arn.is_none() && self.mfa_secret.is_some() {
            return Err("If 'mfa_secret' is set, 'mfa_arn' is required".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Target {
    pub name: String,
    pub source: String,
    pub role_arn: String,
    pub credential_output: CredentialOutputTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_output: Option<CliOutputTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Validation for Target {
    fn validate(&self) -> Result<(), String> {
        if let Some(duration_second) = self.duration_seconds {
            if duration_second < 60 * 15 {
                return Err(format!("validation error in Target(name={}): duration_seconds is 900 (15 minutes) or higher.", self.name));
            }
            if 60 * 60 * 12 < duration_second {
                return Err(format!("Validation Error in Target(name={}): duration_seconds is 43200 (12 hours) or less.", self.name));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Core {
    pub version: String,
    pub save_totp_counter_history: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub core: Core,
    pub source: Vec<Source>,
    pub target: Vec<Target>,
}

impl Validation for Configuration {
    fn validate(&self) -> Result<(), String> {
        let mut source_name_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut target_name_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for source in &self.source {
            if !source_name_set.insert(source.name.to_string()) {
                return Err(format!("Validation Error: name of source must be uniq. (source name '{}' is duplicate.)", source.name));
            }
            source.validate()?;
        }

        for target in &self.target {
            if !target_name_set.insert(target.name.to_string()) {
                return Err(format!("Validation Error: name of target must be uniq. (target name '{}' is duplicate.)", target.name));
            }
            if !source_name_set.contains(&target.source) {
                return Err(format!(
                    "Validation Error in target(name={})): source is not found (source={}).",
                    target.name, target.source
                ));
            }
            target.validate()?;
        }

        Ok(())
    }
}

impl Configuration {
    pub fn new(text: &str) -> Result<Configuration, String> {
        toml::from_str(text).map_err(|e| format!("failed to deserialize configuration: {}", e))
    }

    pub fn to_string(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|e| format!("failed to serialize configuration: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Validation;

    fn make_source() -> Source {
        Source {
            name: "test-source".to_string(),
            profile: None,
            region: None,
            mfa_arn: None,
            mfa_secret: None,
            note: None,
            aws_access_key_id: None,
            aws_secret_access_key: None,
        }
    }

    fn make_target() -> Target {
        Target {
            name: "test-target".to_string(),
            source: "test-source".to_string(),
            role_arn: "arn:aws:iam::123456789012:role/TestRole".to_string(),
            credential_output: CredentialOutputTarget::Json,
            duration_seconds: None,
            region: None,
            cli_output: None,
            note: None,
        }
    }

    fn make_config() -> Configuration {
        Configuration {
            core: Core {
                version: "1".to_string(),
                save_totp_counter_history: None,
            },
            source: vec![make_source()],
            target: vec![make_target()],
        }
    }

    // Source::validate

    #[test]
    fn source_validate_ok_no_static_creds() {
        let s = make_source();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn source_validate_ok_both_static_creds() {
        let s = Source {
            aws_access_key_id: Some("KEY".to_string()),
            aws_secret_access_key: Some("SECRET".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_ok());
    }

    #[test]
    fn source_validate_err_only_access_key() {
        let s = Source {
            aws_access_key_id: Some("KEY".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn source_validate_err_only_secret_key() {
        let s = Source {
            aws_secret_access_key: Some("SECRET".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn source_validate_ok_mfa_arn_and_secret() {
        let s = Source {
            mfa_arn: Some("arn:aws:iam::123456789012:mfa/user".to_string()),
            mfa_secret: Some("TOTP_SECRET".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_ok());
    }

    #[test]
    fn source_validate_err_mfa_secret_without_arn() {
        let s = Source {
            mfa_secret: Some("TOTP_SECRET".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn source_validate_ok_mfa_arn_only() {
        let s = Source {
            mfa_arn: Some("arn:aws:iam::123456789012:mfa/user".to_string()),
            ..make_source()
        };
        assert!(s.validate().is_ok());
    }

    // Target::validate

    #[test]
    fn target_validate_ok_none() {
        let t = make_target();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn target_validate_ok_900() {
        let t = Target { duration_seconds: Some(900), ..make_target() };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn target_validate_err_899() {
        let t = Target { duration_seconds: Some(899), ..make_target() };
        assert!(t.validate().is_err());
    }

    #[test]
    fn target_validate_ok_43200() {
        let t = Target { duration_seconds: Some(43200), ..make_target() };
        assert!(t.validate().is_ok());
    }

    #[test]
    fn target_validate_err_43201() {
        let t = Target { duration_seconds: Some(43201), ..make_target() };
        assert!(t.validate().is_err());
    }

    // Configuration::validate

    #[test]
    fn config_validate_ok() {
        assert!(make_config().validate().is_ok());
    }

    #[test]
    fn config_validate_err_duplicate_source_name() {
        let mut config = make_config();
        config.source.push(make_source());
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validate_err_duplicate_target_name() {
        let mut config = make_config();
        config.target.push(make_target());
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_validate_err_target_source_not_found() {
        let mut config = make_config();
        config.target[0].source = "nonexistent".to_string();
        assert!(config.validate().is_err());
    }

    // CredentialOutputTarget::new

    #[test]
    fn credential_output_target_new_json() {
        assert_eq!(CredentialOutputTarget::new("json"), Ok(CredentialOutputTarget::Json));
        assert_eq!(CredentialOutputTarget::new("j"), Ok(CredentialOutputTarget::Json));
        assert_eq!(CredentialOutputTarget::new("Json"), Ok(CredentialOutputTarget::Json));
    }

    #[test]
    fn credential_output_target_new_bash() {
        assert_eq!(CredentialOutputTarget::new("bash"), Ok(CredentialOutputTarget::Bash));
        assert_eq!(CredentialOutputTarget::new("b"), Ok(CredentialOutputTarget::Bash));
    }

    #[test]
    fn credential_output_target_new_fish() {
        assert_eq!(CredentialOutputTarget::new("fish"), Ok(CredentialOutputTarget::Fish));
        assert_eq!(CredentialOutputTarget::new("f"), Ok(CredentialOutputTarget::Fish));
    }

    #[test]
    fn credential_output_target_new_powershell() {
        assert_eq!(CredentialOutputTarget::new("powershell"), Ok(CredentialOutputTarget::PowerShell));
        assert_eq!(CredentialOutputTarget::new("p"), Ok(CredentialOutputTarget::PowerShell));
    }

    #[test]
    fn credential_output_target_new_shared_credentials() {
        assert_eq!(CredentialOutputTarget::new("sharedcredentials"), Ok(CredentialOutputTarget::SharedCredentials));
        assert_eq!(CredentialOutputTarget::new("s"), Ok(CredentialOutputTarget::SharedCredentials));
    }

    #[test]
    fn credential_output_target_new_invalid() {
        assert!(CredentialOutputTarget::new("invalid").is_err());
    }

    // CliOutputTarget Display

    #[test]
    fn cli_output_target_display() {
        assert_eq!(CliOutputTarget::Json.to_string(), "json");
        assert_eq!(CliOutputTarget::Yaml.to_string(), "yaml");
        assert_eq!(CliOutputTarget::YamlStream.to_string(), "yaml-stream");
        assert_eq!(CliOutputTarget::Text.to_string(), "text");
        assert_eq!(CliOutputTarget::Table.to_string(), "table");
    }
}
