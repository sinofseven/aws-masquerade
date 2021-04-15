use crate::lib::dirs::MASQUERADE_PATH;
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::{
    collections::{BTreeMap, HashMap},
    str,
};

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
    pub fn to_str(&self) -> &str {
        match self {
            CredentialOutputTarget::Bash => "bash",
            CredentialOutputTarget::Fish => "fish",
            CredentialOutputTarget::PowerShell => "PowerShell",
            CredentialOutputTarget::SharedCredentials => "SharedCredentials",
        }
    }

    pub fn from_str(v: &str) -> Result<CredentialOutputTarget, String> {
        match v {
            "bash" => Ok(CredentialOutputTarget::Bash),
            "fish" => Ok(CredentialOutputTarget::Fish),
            "PowerShell" => Ok(CredentialOutputTarget::PowerShell),
            "SharedCredentials" => Ok(CredentialOutputTarget::SharedCredentials),
            _ => Err("Invalid Name of CredentialOutputTarget".to_string()),
        }
    }
}

impl Display for CredentialOutputTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
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
    pub fn to_str(&self) -> &str {
        match self {
            AwsCliOutput::Json => "json",
            AwsCliOutput::Text => "text",
            AwsCliOutput::Table => "table",
        }
    }

    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}

impl Display for AwsCliOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    // assume setting
    pub source_profile: Option<String>,
    pub role_arn: String,
    pub mfa_arn: Option<String>,
    pub mfa_secret: Option<String>,
    // output setting
    pub credential_output: CredentialOutputTarget,
    pub output: Option<AwsCliOutput>,
    #[serde(with = "ext_region")]
    pub region: Option<Region>,
}

impl Account {
    pub fn create_shared_config(&self) -> Option<HashMap<String, String>> {
        let mut map: HashMap<String, String> = HashMap::new();

        if let Some(region) = &self.region {
            map.insert("region".to_string(), region.name().to_string());
        }

        if let Some(output) = &self.output {
            map.insert("output".to_string(), output.to_string());
        }

        if map.len() > 0 {
            Some(map)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MasqueradeConfig {
    pub accounts: BTreeMap<String, Account>,
}

impl MasqueradeConfig {
    pub fn new() -> MasqueradeConfig {
        MasqueradeConfig {
            accounts: BTreeMap::new(),
        }
    }
}

mod ext_region {
    use rusoto_core::Region;
    use serde::de::Unexpected;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Option<Region>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => serializer.serialize_none(),
            Some(region) => serializer.serialize_str(region.name()),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Region>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let region: Option<String> = Option::deserialize(deserializer)?;
        if let Some(region) = region {
            match Region::from_str(&region) {
                Ok(region) => Ok(Some(region)),
                Err(_) => Err(Error::invalid_value(
                    Unexpected::Str(&region),
                    &"invalid region name",
                )),
            }
        } else {
            Ok(None)
        }
    }
}

pub fn load_config() -> Result<MasqueradeConfig, String> {
    let config_path = MASQUERADE_PATH.config();
    let text = match std::fs::read_to_string(config_path) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to read config: {}", e)),
    };
    match serde_json::from_str(&text) {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("failed to parse config: {}", e)),
    }
}

pub fn save_config(config: &MasqueradeConfig) -> Result<(), String> {
    let config_path = MASQUERADE_PATH.config();
    let dir = config_path.parent().unwrap();
    match std::fs::create_dir_all(dir) {
        Ok(_) => (),
        Err(e) => return Err(format!("failed to create config directory: {}", e)),
    };
    let text = match serde_json::to_string_pretty(config) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to serialize config: {}", e)),
    };
    match std::fs::write(config_path, text) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("failed to write config: {}", e)),
    }
}

pub fn load_shared_config() -> Result<HashMap<String, HashMap<String, String>>, String> {
    let path_shared_config = MASQUERADE_PATH.shared_config();
    let text = match std::fs::read_to_string(path_shared_config) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to read shared config: {}", e)),
    };
    match serde_ini::from_str(&text) {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("failed to parse shared config: {}", e)),
    }
}

pub fn save_shared_config(config: &HashMap<String, HashMap<String, String>>) -> Result<(), String> {
    let path_shared_config = MASQUERADE_PATH.shared_config();
    let path_dir = match path_shared_config.parent() {
        Some(path) => path,
        None => return Err("failed to resolve directory of shared config".to_string()),
    };
    match std::fs::create_dir_all(path_dir) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!(
                "failed to create directories of shared config: {}",
                e
            ))
        }
    };

    let text = match serde_ini::to_string(config) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to serialize shared config: {}", e)),
    };
    match std::fs::write(path_shared_config, text) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("failed to write shared config: {}", e)),
    }
}

pub fn add_shared_config<T>(name: T, data: &HashMap<String, String>) -> Result<(), String>
where
    T: Display,
{
    let mut config = load_shared_config()?;
    let profile_name = format!("profile {}", name);

    config.insert(profile_name, data.clone());

    save_shared_config(&config)
}

pub fn load_shared_credentials() -> Result<HashMap<String, HashMap<String, String>>, String> {
    let path_shred_credentials = MASQUERADE_PATH.shared_credentials();
    let text = match std::fs::read_to_string(path_shred_credentials) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to read shared credentials: {}", e)),
    };
    match serde_ini::from_str(&text) {
        Ok(credentials) => Ok(credentials),
        Err(e) => Err(format!("failed to deserialize shared credentials: {}", e)),
    }
}

pub fn save_shared_credentials(
    credentials: &HashMap<String, HashMap<String, String>>,
) -> Result<(), String> {
    let path_shared_credentials = MASQUERADE_PATH.shared_credentials();
    let path_dir = match path_shared_credentials.parent() {
        Some(path) => path,
        None => return Err("failed to resolve directory of shared credentials".to_string()),
    };
    match std::fs::create_dir_all(path_dir) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!(
                "failed to create directories of shared credentials: {}",
                e
            ))
        }
    };

    let text = match serde_ini::to_string(credentials) {
        Ok(text) => text,
        Err(e) => return Err(format!("failed to serialize shared credentials: {}", e)),
    };
    match std::fs::write(path_shared_credentials, text) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("failed to write shared credentials: {}", e)),
    }
}

pub fn add_into_shared_credentials(
    name: &String,
    data: &HashMap<String, String>,
) -> Result<(), String> {
    let mut credentials = load_shared_credentials()?;

    credentials.insert(name.clone(), data.clone());

    save_shared_credentials(&credentials)
}
