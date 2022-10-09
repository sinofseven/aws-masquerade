use crate::base::Validation;
use crate::models::configuration::v1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum CredentialOutputTarget {
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "fish")]
    Fish,
    PowerShell,
    SharedCredentials,
}

impl CredentialOutputTarget {
    fn migrate(&self) -> v1::CredentialOutputTarget {
        match self {
            CredentialOutputTarget::Bash => v1::CredentialOutputTarget::Bash,
            CredentialOutputTarget::Fish => v1::CredentialOutputTarget::Fish,
            CredentialOutputTarget::PowerShell => v1::CredentialOutputTarget::PowerShell,
            CredentialOutputTarget::SharedCredentials => {
                v1::CredentialOutputTarget::SharedCredentials
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AwsCliOutput {
    Json,
    Text,
    Table,
}

impl AwsCliOutput {
    fn migrate(&self) -> v1::CliOutputTarget {
        match self {
            AwsCliOutput::Json => v1::CliOutputTarget::Json,
            AwsCliOutput::Text => v1::CliOutputTarget::Text,
            AwsCliOutput::Table => v1::CliOutputTarget::Table,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub source_profile: Option<String>,
    pub role_arn: String,
    pub mfa_arn: Option<String>,
    pub mfa_secret: Option<String>,
    pub credential_output: CredentialOutputTarget,
    pub output: Option<AwsCliOutput>,
    pub region: Option<String>,
}

impl Account {
    fn migrate(&self, name: &str) -> (v1::Source, v1::Target) {
        let profile = if let Some(profile) = &self.source_profile {
            profile
        } else {
            name
        };
        let source = v1::Source {
            name: profile.to_string(),
            from_environment: match self.source_profile {
                Some(_) => None,
                None => Some(true),
            },
            profile: self.source_profile.clone(),
            credentials: None,
            region: None,
            mfa_arn: self.mfa_arn.clone(),
            mfa_secret: self.mfa_secret.clone(),
            note: None,
        };
        let target = v1::Target {
            name: name.to_string(),
            source: profile.to_string(),
            role_arn: self.role_arn.clone(),
            credential_output: self.credential_output.migrate(),
            duration_seconds: None,
            region: self.region.clone(),
            cli_output: self.output.clone().map(|output| output.migrate()),
            note: None,
        };

        (source, target)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MasqueradeConfig {
    pub accounts: std::collections::BTreeMap<String, Account>,
}

impl Validation for MasqueradeConfig {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MasqueradeConfig {
    pub fn new(text: &str) -> Result<MasqueradeConfig, String> {
        serde_json::from_str(text)
            .map_err(|e| format!("failed to deserialize configuration: {}", e))
    }

    pub fn to_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to seralize configuration: {}", e))
    }

    pub fn migrate(&self) -> v1::Configuration {
        let mut set_source: std::collections::HashSet<v1::Source> =
            std::collections::HashSet::new();
        let mut set_target: std::collections::HashSet<v1::Target> =
            std::collections::HashSet::new();

        for (name, account) in &self.accounts {
            let (source, target) = account.migrate(name);
            set_source.insert(source);
            set_target.insert(target);
        }

        v1::Configuration {
            source: set_source.into_iter().collect(),
            target: set_target.into_iter().collect(),
            core: v1::Core {
                version: "1.0".to_string(),
            },
        }
    }
}
