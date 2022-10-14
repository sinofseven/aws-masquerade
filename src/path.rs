use crate::variables::cmd::configure as cmd_configure;
use crate::variables::models::configuration::Version as ConfigVersion;
use colored::Colorize;
use dirs::home_dir;
use std::env;
use std::path::PathBuf;

const FAILD_RESOLVE_HOME: &str = "failed to resolve Home direcotry";

pub fn get_path_aws_shared_credentials() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("AWS_SHARED_CREDENTIALS_FILE") {
        return Ok(PathBuf::from(path));
    }
    home_dir()
        .map(|home| home.join(".aws/credentials"))
        .ok_or_else(|| FAILD_RESOLVE_HOME.to_string())
}

pub fn get_path_aws_config() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("AWS_CONFIG_FILE") {
        return Ok(PathBuf::from(path));
    }
    home_dir()
        .map(|home| home.join(".aws/config"))
        .ok_or_else(|| FAILD_RESOLVE_HOME.to_string())
}

pub fn get_path_masquerade_config() -> Result<PathBuf, String> {
    home_dir()
        .map(|home| home.join(".config/aws-masquerade/config.toml"))
        .ok_or_else(|| FAILD_RESOLVE_HOME.to_string())
}

pub fn get_path_old_masquerade_config() -> Result<PathBuf, String> {
    home_dir()
        .map(|h| h.join(".config/aws-masquerade/config.json"))
        .ok_or_else(|| FAILD_RESOLVE_HOME.to_string())
}

pub fn get_current_path_masquerade_config() -> Result<(PathBuf, ConfigVersion), String> {
    let old = get_path_old_masquerade_config()?;
    let latest = get_path_masquerade_config()?;

    if !latest.exists() && old.exists() {
        eprintln!("\n{}", "there is old configuration file.".yellow());
        eprintln!(
            "{}",
            format!("please create {} or run next command", latest.display()).yellow()
        );
        eprintln!(
            "{}\n",
            format!(
                "$ aws-masquerade {} {}",
                cmd_configure::NAME,
                cmd_configure::sub_command::MIGRATE
            )
            .yellow()
        );
        Ok((old, ConfigVersion::V0))
    } else {
        Ok((latest, ConfigVersion::V1))
    }
}

pub fn get_path_totp_generate_history() -> Result<PathBuf, String> {
    home_dir()
        .map(|h| h.join(".config/aws-masquerade/.totp_count_history.json"))
        .ok_or_else(|| FAILD_RESOLVE_HOME.to_string())
}
