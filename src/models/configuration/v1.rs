use crate::base::Validation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CredentialOutputTarget {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "fish")]
    Fish,
    PowerShell,
    SharedCredentials,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Credential {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Source {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_environment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Credential>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Validation for Source {
    fn validate(&self) -> Result<(), String> {
        if self.from_environment.is_none() && self.profile.is_none() && self.credentials.is_none() {
            return Err(format!("Validation Error in Source(name={}): One of ['from_environment', 'profile', 'credentials'] is required.", self.name));
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
    pub duration_seconds: Option<u32>,
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub source: Vec<Source>,
    pub target: Vec<Target>,
    pub core: Core,
}

impl Validation for Configuration {
    fn validate(&self) -> Result<(), String> {
        let mut source_name_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut target_name_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for source in &self.source {
            if !source_name_set.insert(source.name.to_string()) {
                return Err(format!("Validation Error: name of source must be uniq. (tource name '{}' is dupplicate.)", source.name));
            }
            source.validate()?;
        }

        for target in &self.target {
            if !target_name_set.insert(target.name.to_string()) {
                return Err(format!("Validation Error: name of target must be uniq. (tource name '{}' is dupplicate.)", target.name));
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
        toml::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize configuration: {}", e))
    }
}
