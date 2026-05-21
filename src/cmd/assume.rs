use crate::base::Validation;
use crate::models::configuration::v1;
use crate::models::configuration::v1::{CliOutputTarget, CredentialOutputTarget};
use crate::variables::output::environment_variables as env;
use crate::variables::output::shared_credentials;
use aws_sdk_sts::config::Credentials;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(clap::Args)]
#[command(about = "execute assume role", arg_required_else_help = true)]
pub struct AssumeArgs {
    target_name: String,
    #[arg(
        short = 'c',
        long = "credential-output",
        value_name = "CREDENTIAL_OUTPUT",
        help = "output of assume role result"
    )]
    credential_output: Option<CredentialOutputTarget>,
}

impl AssumeArgs {
    pub fn run(self) -> Result<(), String> {
        let config = crate::models::configuration::load_configuration()?;
        config.validate()?;
        let is_save_totp_last_counter = config.core.save_totp_counter_history.unwrap_or(false);

        let target = config
            .target
            .iter()
            .find(|t| t.name == self.target_name)
            .ok_or_else(|| format!("target(name={}) is not found.", self.target_name))?;
        let source = config
            .source
            .iter()
            .find(|s| s.name == target.source)
            .ok_or_else(|| format!("source(name={} is not found.", &target.source))?;

        let credential_output = self
            .credential_output
            .as_ref()
            .unwrap_or(&target.credential_output);

        let resp = tokio::runtime::Runtime::new()
            .map_err(|e| format!("failed to create async runtime: {}", e))?
            .block_on(exec_assume(source, target, &is_save_totp_last_counter))
            .map_err(|e| format!("failed to execute assume role: {}", e))?;

        exec_output(
            credential_output,
            &target.region,
            &target.cli_output,
            &target.note,
            &target.name,
            &resp,
        )?;

        Ok(())
    }
}

pub(crate) fn make_static_credentials(
    access_key_id: &str,
    secret_access_key: &str,
) -> Credentials {
    Credentials::new(access_key_id, secret_access_key, None, None, "Static")
}

pub(crate) fn build_region_provider(
    region: Option<String>,
) -> aws_config::meta::region::RegionProviderChain {
    aws_config::meta::region::RegionProviderChain::first_try(
        region.map(aws_types::region::Region::new),
    )
    .or_default_provider()
    .or_else(aws_types::region::Region::new("us-east-1"))
}

async fn generate_sdk_config(source: &v1::Source) -> aws_config::SdkConfig {
    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());

    if let (Some(aws_access_key), Some(aws_secret_access_key)) =
        (&source.aws_access_key_id, &source.aws_secret_access_key)
    {
        config_loader = config_loader
            .credentials_provider(make_static_credentials(aws_access_key, aws_secret_access_key));
    }

    if let Some(profile) = &source.profile {
        config_loader = config_loader.profile_name(profile);
    }

    config_loader = config_loader.region(build_region_provider(source.region.clone()));

    config_loader.load().await
}

async fn exec_assume(
    source: &v1::Source,
    target: &v1::Target,
    is_save_totp_last_counter: &bool,
) -> Result<aws_sdk_sts::operation::assume_role::AssumeRoleOutput, String> {
    let sdk_config = generate_sdk_config(source).await;
    let client = aws_sdk_sts::Client::new(&sdk_config);

    let mut assume_role = client
        .assume_role()
        .role_session_name(format!("session-{}", uuid::Uuid::new_v4()))
        .role_arn(&target.role_arn);

    if let Some(duration_seconds) = &target.duration_seconds {
        assume_role = assume_role.duration_seconds(i32::from(*duration_seconds));
    }

    if let Some(mfa_arn) = &source.mfa_arn {
        assume_role = assume_role.serial_number(mfa_arn);
        let token = &source.mfa_secret.as_ref().map_or_else(
            || Ok(crate::io::get_input("\nMFA TOKEN: ")),
            |secret| crate::totp::generate(secret, is_save_totp_last_counter),
        )?;
        assume_role = assume_role.token_code(token);
    }

    assume_role
        .send()
        .await
        .map_err(|e| format!("failed to assume role: {e:#?}"))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct JsonCredential {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
    pub(crate) session_token: String,
}

pub(crate) fn build_json_output(credential: &JsonCredential) -> Result<String, String> {
    serde_json::to_string_pretty(credential)
        .map_err(|e| format!("failed to serialize assume role result: {}", e))
}

pub(crate) fn build_bash_output(
    credential: &JsonCredential,
    region: Option<&str>,
    cli_output: Option<&CliOutputTarget>,
    note: Option<&str>,
    invocation: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "export {}=\"{}\"",
        env::AWS_ACCESS_KEY_ID,
        credential.access_key_id
    ));
    lines.push(format!(
        "export {}=\"{}\"",
        env::AWS_SECRET_ACCESS_KEY,
        credential.secret_access_key
    ));
    lines.push(format!(
        "export {}=\"{}\"",
        env::AWS_SESSION_TOKEN,
        credential.session_token
    ));
    lines.push(format!(
        "export {}=\"{}\"",
        env::AWS_SECURITY_TOKEN,
        credential.session_token
    ));

    if let Some(region) = region {
        lines.push(format!("export {}=\"{}\"", env::AWS_DEFAULT_REGION, region));
        lines.push(format!("export {}=\"{}\"", env::AWS_REGION, region));
    }

    if let Some(cli_output) = cli_output {
        lines.push(format!(
            "export {}=\"{}\"",
            env::AWS_DEFAULT_OUTPUT,
            cli_output
        ));
    }

    if let Some(note) = note {
        for (i, note_line) in note.split('\n').enumerate() {
            let prefix = match i {
                0 => "note: ",
                _ => " ",
            };
            lines.push(format!("# {}{}", prefix, note_line))
        }
    }

    lines.push("# Run this to configure your shell:".to_string());
    lines.push(format!("# eval $({})", invocation));
    lines.join("\n")
}

pub(crate) fn build_fish_output(
    credential: &JsonCredential,
    region: Option<&str>,
    cli_output: Option<&CliOutputTarget>,
    note: Option<&str>,
    invocation: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "set -gx {} \"{}\"",
        env::AWS_ACCESS_KEY_ID,
        credential.access_key_id
    ));
    lines.push(format!(
        "set -gx {} \"{}\"",
        env::AWS_SECRET_ACCESS_KEY,
        credential.secret_access_key
    ));
    lines.push(format!(
        "set -gx {} \"{}\"",
        env::AWS_SESSION_TOKEN,
        credential.session_token
    ));
    lines.push(format!(
        "set -gx {} \"{}\"",
        env::AWS_SECURITY_TOKEN,
        credential.session_token
    ));

    if let Some(region) = region {
        lines.push(format!(
            "set -gx {} \"{}\"",
            env::AWS_DEFAULT_REGION,
            region
        ));
        lines.push(format!("set -gx {} \"{}\"", env::AWS_REGION, region));
    }

    if let Some(cli_output) = cli_output {
        lines.push(format!(
            "set -gx {} \"{}\"",
            env::AWS_DEFAULT_OUTPUT,
            cli_output
        ));
    }

    if let Some(note) = note {
        for (i, note_line) in note.split('\n').enumerate() {
            let prefix = match i {
                0 => "note: ",
                _ => " ",
            };
            lines.push(format!("# {}{}", prefix, note_line))
        }
    }

    lines.push("# Run this to configure your shell:".to_string());
    lines.push(format!("# {} | source", invocation));

    lines.join("\n")
}

pub(crate) fn build_powershell_output(
    credential: &JsonCredential,
    region: Option<&str>,
    cli_output: Option<&CliOutputTarget>,
    note: Option<&str>,
    invocation: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!(
        "$env:{}=\"{}\"",
        env::AWS_ACCESS_KEY_ID,
        credential.access_key_id
    ));
    lines.push(format!(
        "$env:{}=\"{}\"",
        env::AWS_SECRET_ACCESS_KEY,
        credential.secret_access_key
    ));
    lines.push(format!(
        "$env:{}=\"{}\"",
        env::AWS_SESSION_TOKEN,
        credential.session_token
    ));
    lines.push(format!(
        "$env:{}=\"{}\"",
        env::AWS_SECURITY_TOKEN,
        credential.session_token
    ));

    if let Some(region) = region {
        lines.push(format!("$env:{}=\"{}\"", env::AWS_DEFAULT_REGION, region));
        lines.push(format!("$env:{}=\"{}\"", env::AWS_REGION, region));
    }

    if let Some(cli_output) = cli_output {
        lines.push(format!(
            "$env:{}=\"{}\"",
            env::AWS_DEFAULT_OUTPUT,
            cli_output
        ));
    }

    if let Some(note) = note {
        for (i, note_line) in note.split('\n').enumerate() {
            let prefix = match i {
                0 => "note: ",
                _ => " ",
            };
            lines.push(format!("# {}{}", prefix, note_line));
        }
    }

    lines.push("# Run this to configure your shell:".to_string());
    lines.push(format!("# {} | Invoke-Expression", invocation));

    lines.join("\n")
}

fn exec_output(
    output_target: &CredentialOutputTarget,
    region: &Option<String>,
    cli_output: &Option<CliOutputTarget>,
    note: &Option<String>,
    name: &str,
    output_assume_role: &aws_sdk_sts::operation::assume_role::AssumeRoleOutput,
) -> Result<(), String> {
    let invocation = {
        let args: Vec<String> = std::env::args().collect();
        args.join(" ")
    };
    let credential_raw = output_assume_role
        .credentials()
        .ok_or_else(|| "there is not credentials in assume role result.".to_string())?;
    let credential = JsonCredential {
        access_key_id: credential_raw.access_key_id().to_string(),
        secret_access_key: credential_raw.secret_access_key().to_string(),
        session_token: credential_raw.session_token().to_string(),
    };

    let text = match output_target {
        CredentialOutputTarget::Json => build_json_output(&credential)?,
        CredentialOutputTarget::Bash => build_bash_output(
            &credential,
            region.as_deref(),
            cli_output.as_ref(),
            note.as_deref(),
            &invocation,
        ),
        CredentialOutputTarget::Fish => build_fish_output(
            &credential,
            region.as_deref(),
            cli_output.as_ref(),
            note.as_deref(),
            &invocation,
        ),
        CredentialOutputTarget::PowerShell => build_powershell_output(
            &credential,
            region.as_deref(),
            cli_output.as_ref(),
            note.as_deref(),
            &invocation,
        ),
        CredentialOutputTarget::SharedCredentials => {
            let path = crate::path::get_path_aws_shared_credentials()?;
            let mut configure: BTreeMap<String, BTreeMap<String, String>> = if path.exists() {
                let text = crate::fs::load_text(&path)?;
                serde_ini::from_str(&text).map_err(|e| {
                    format!("failed to deserialize aws shared credential file: {}", e)
                })?
            } else {
                BTreeMap::new()
            };

            let mut profile = match configure.get(name) {
                Some(profile) => profile.clone(),
                None => BTreeMap::new(),
            };

            profile.insert(
                shared_credentials::AWS_ACCESS_KEY_ID.to_string(),
                credential.access_key_id,
            );
            profile.insert(
                shared_credentials::AWS_SECRET_ACCESS_KEY.to_string(),
                credential.secret_access_key,
            );
            profile.insert(
                shared_credentials::AWS_SESSION_TOKEN.to_string(),
                credential.session_token.clone(),
            );
            profile.insert(
                shared_credentials::AWS_SECURITY_TOKEN.to_string(),
                credential.session_token,
            );

            {
                let expires = credential_raw.expiration();
                let datetime =
                    chrono::DateTime::from_timestamp(expires.secs(), expires.subsec_nanos())
                        .unwrap_or_default();
                profile.insert(
                    shared_credentials::X_SECURITY_TOKEN_EXPIRES.to_string(),
                    datetime.to_rfc3339(),
                );
            }

            if let Some(assumed_role_user) = output_assume_role.assumed_role_user() {
                let arn = assumed_role_user.arn();
                profile.insert(
                    shared_credentials::X_PRINCIPAL_ARN.to_string(),
                    arn.to_string(),
                );
            }

            if let Some(region) = region {
                profile.insert(shared_credentials::REGION.to_string(), region.to_string());
            }

            if let Some(output) = cli_output {
                profile.insert(shared_credentials::OUTPUT.to_string(), output.to_string());
            }

            configure.insert(name.to_string(), profile);

            let text = serde_ini::to_string(&configure)
                .map_err(|e| format!("failed to serialize aws shared credentials file: {}", e))?;
            crate::fs::save_text(&path, &text)?;

            let mut lines: Vec<String> = Vec::new();

            lines.push(
                "Your new access key pair has been stored in the AWS configuration".to_string(),
            );
            lines.push(format!("To use this credential, call the AWS CLI with the --profile option (e.g. aws sts get-caller-identity --profile {})", name));

            lines.join("\n")
        }
    };

    println!("{}", text);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_credential() -> JsonCredential {
        JsonCredential {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: "SESSION_TOKEN".to_string(),
        }
    }

    // make_static_credentials

    #[test]
    fn credentials_access_key_and_secret_are_set() {
        let creds = make_static_credentials("ACCESS_KEY", "SECRET_KEY");
        assert_eq!(creds.access_key_id(), "ACCESS_KEY");
        assert_eq!(creds.secret_access_key(), "SECRET_KEY");
    }

    // build_region_provider

    #[tokio::test]
    async fn region_provider_returns_explicit_region() {
        let provider = build_region_provider(Some("us-west-2".to_string()));
        let region = provider.region().await.unwrap();
        assert_eq!(region.as_ref(), "us-west-2");
    }

    // build_json_output

    #[test]
    fn build_json_ok() {
        let cred = make_credential();
        let json = build_json_output(&cred).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("access_key_id").is_some());
        assert!(value.get("secret_access_key").is_some());
        assert!(value.get("session_token").is_some());
    }

    // build_bash_output

    #[test]
    fn build_bash_minimal() {
        let cred = make_credential();
        let output = build_bash_output(&cred, None, None, None, "aws-masquerade assume target");
        assert!(output.contains("export AWS_ACCESS_KEY_ID=\"AKIAIOSFODNN7EXAMPLE\""));
        assert!(output.contains("export AWS_SECRET_ACCESS_KEY=\"wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\""));
        assert!(output.contains("export AWS_SESSION_TOKEN=\"SESSION_TOKEN\""));
        assert!(output.contains("export AWS_SECURITY_TOKEN=\"SESSION_TOKEN\""));
        assert!(output.contains("# eval $(aws-masquerade assume target)"));
        assert!(!output.contains("AWS_DEFAULT_REGION"));
        assert!(!output.contains("AWS_DEFAULT_OUTPUT"));
    }

    #[test]
    fn build_bash_with_region() {
        let cred = make_credential();
        let output = build_bash_output(&cred, Some("ap-northeast-1"), None, None, "cmd");
        assert!(output.contains("export AWS_DEFAULT_REGION=\"ap-northeast-1\""));
        assert!(output.contains("export AWS_REGION=\"ap-northeast-1\""));
    }

    #[test]
    fn build_bash_with_cli_output() {
        let cred = make_credential();
        let output = build_bash_output(&cred, None, Some(&CliOutputTarget::Json), None, "cmd");
        assert!(output.contains("export AWS_DEFAULT_OUTPUT=\"json\""));
    }

    #[test]
    fn build_bash_with_single_line_note() {
        let cred = make_credential();
        let output = build_bash_output(&cred, None, None, Some("test note"), "cmd");
        assert!(output.contains("# note: test note"));
    }

    #[test]
    fn build_bash_with_multiline_note() {
        let cred = make_credential();
        let output = build_bash_output(&cred, None, None, Some("line1\nline2"), "cmd");
        assert!(output.contains("# note: line1"));
        assert!(output.contains("#  line2"));
    }

    #[test]
    fn build_bash_with_all_options() {
        let cred = make_credential();
        let output = build_bash_output(
            &cred,
            Some("eu-west-1"),
            Some(&CliOutputTarget::Table),
            Some("my note"),
            "aws-masquerade assume target",
        );
        assert!(output.contains("export AWS_ACCESS_KEY_ID="));
        assert!(output.contains("export AWS_DEFAULT_REGION=\"eu-west-1\""));
        assert!(output.contains("export AWS_REGION=\"eu-west-1\""));
        assert!(output.contains("export AWS_DEFAULT_OUTPUT=\"table\""));
        assert!(output.contains("# note: my note"));
        assert!(output.contains("# eval $(aws-masquerade assume target)"));
    }

    // build_fish_output

    #[test]
    fn build_fish_minimal() {
        let cred = make_credential();
        let output = build_fish_output(&cred, None, None, None, "aws-masquerade assume target");
        assert!(output.contains("set -gx AWS_ACCESS_KEY_ID \"AKIAIOSFODNN7EXAMPLE\""));
        assert!(output.contains("set -gx AWS_SECRET_ACCESS_KEY \"wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\""));
        assert!(output.contains("set -gx AWS_SESSION_TOKEN \"SESSION_TOKEN\""));
        assert!(output.contains("set -gx AWS_SECURITY_TOKEN \"SESSION_TOKEN\""));
        assert!(output.contains("# aws-masquerade assume target | source"));
        assert!(!output.contains("AWS_DEFAULT_REGION"));
    }

    #[test]
    fn build_fish_with_region() {
        let cred = make_credential();
        let output = build_fish_output(&cred, Some("ap-northeast-1"), None, None, "cmd");
        assert!(output.contains("set -gx AWS_DEFAULT_REGION \"ap-northeast-1\""));
        assert!(output.contains("set -gx AWS_REGION \"ap-northeast-1\""));
    }

    #[test]
    fn build_fish_with_note() {
        let cred = make_credential();
        let output = build_fish_output(&cred, None, None, Some("fish note"), "cmd");
        assert!(output.contains("# note: fish note"));
    }

    // build_powershell_output

    #[test]
    fn build_powershell_minimal() {
        let cred = make_credential();
        let output =
            build_powershell_output(&cred, None, None, None, "aws-masquerade assume target");
        assert!(output.contains("$env:AWS_ACCESS_KEY_ID=\"AKIAIOSFODNN7EXAMPLE\""));
        assert!(output.contains("$env:AWS_SECRET_ACCESS_KEY=\"wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\""));
        assert!(output.contains("$env:AWS_SESSION_TOKEN=\"SESSION_TOKEN\""));
        assert!(output.contains("$env:AWS_SECURITY_TOKEN=\"SESSION_TOKEN\""));
        assert!(output.contains("# aws-masquerade assume target | Invoke-Expression"));
        assert!(!output.contains("AWS_DEFAULT_REGION"));
    }

    #[test]
    fn build_powershell_with_region() {
        let cred = make_credential();
        let output = build_powershell_output(&cred, Some("us-east-1"), None, None, "cmd");
        assert!(output.contains("$env:AWS_DEFAULT_REGION=\"us-east-1\""));
        assert!(output.contains("$env:AWS_REGION=\"us-east-1\""));
    }
}
