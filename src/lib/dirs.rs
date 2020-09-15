use dirs;
use std::path::{Path, PathBuf};

pub struct MasqueradePath {
    config_file: PathBuf,
    shared_credential_file: PathBuf,
    shared_config_file: PathBuf,
}

fn get_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".config/aws-masquerade/config.json"))
}

fn get_shared_credential_path() -> Option<PathBuf> {
    std::env::var("AWS_SHARED_CREDENTIALS_FILE")
        .ok()
        .map(PathBuf::from)
        .filter(|f| f.is_absolute())
        .or_else(|| dirs::home_dir().map(|d| d.join(".aws/credentials")))
}

fn get_shared_config_path() -> Option<PathBuf> {
    std::env::var("AWS_CONFIG_FILE")
        .ok()
        .map(PathBuf::from)
        .filter(|f| f.is_absolute())
        .or_else(|| dirs::home_dir().map(|d| d.join(".aws/config")))
}

impl MasqueradePath {
    fn new() -> Option<MasqueradePath> {
        let config = get_config_path()?;
        let shared_credential = get_shared_credential_path()?;
        let shared_config = get_shared_config_path()?;
        Some(MasqueradePath {
            config_file: config,
            shared_credential_file: shared_credential,
            shared_config_file: shared_config,
        })
    }

    pub fn config(&self) -> &Path {
        &self.config_file
    }
    pub fn shared_credentials(&self) -> &Path {
        &self.shared_credential_file
    }
    pub fn shared_config(&self) -> &Path {
        &self.shared_config_file
    }
}

lazy_static! {
    pub static ref MASQUERADE_PATH: MasqueradePath =
        MasqueradePath::new().expect("Could not get aws-masquerade paths");
}
