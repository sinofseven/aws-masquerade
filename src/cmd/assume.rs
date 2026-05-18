use crate::base::Validation;
use crate::models::configuration::v1;
use crate::models::configuration::v1::{CliOutputTarget, CredentialOutputTarget};
use crate::variables::output::environment_variables as env;
use crate::variables::output::shared_credentials;
use aws_sdk_sts::config::Credentials;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(clap::Args)]
#[command(about = "execute assume role")]
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

async fn generate_sdk_config(source: &v1::Source) -> aws_config::SdkConfig {
    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
    if let (Some(aws_access_key), Some(aws_secret_access_key)) =
        (&source.aws_access_key_id, &source.aws_secret_access_key)
    {
        let credential_provider =
            Credentials::new(aws_access_key, aws_secret_access_key, None, None, "Static");
        config_loader = config_loader.credentials_provider(credential_provider);
    }
    if let Some(profile) = &source.profile {
        let credential_provider = aws_config::profile::ProfileFileCredentialsProvider::builder()
            .profile_name(profile)
            .build();
        config_loader = config_loader.credentials_provider(credential_provider);
    }
    let region = source.region.clone().map_or_else(
        || aws_types::region::Region::new("us-east-1"),
        aws_types::region::Region::new,
    );

    config_loader.region(region).load().await
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

#[derive(Debug, Clone, Serialize)]
struct JsonCredential {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
}

fn exec_output(
    output_target: &CredentialOutputTarget,
    region: &Option<String>,
    cli_output: &Option<CliOutputTarget>,
    note: &Option<String>,
    name: &str,
    output_assume_role: &aws_sdk_sts::operation::assume_role::AssumeRoleOutput,
) -> Result<(), String> {
    let args = {
        let args: Vec<String> = std::env::args().collect();
        args.join(" ")
    };
    let credential = output_assume_role
        .credentials()
        .ok_or_else(|| "there is not credentials in assume role result.".to_string())?;
    let model = JsonCredential {
        access_key_id: credential.access_key_id().to_string(),
        secret_access_key: credential.secret_access_key().to_string(),
        session_token: credential.session_token().to_string(),
    };
    let text = match output_target {
        CredentialOutputTarget::Json => serde_json::to_string_pretty(&model)
            .map_err(|e| format!("failed to serialize assume role result: {}", e))?,
        CredentialOutputTarget::Bash => {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!(
                "export {}=\"{}\"",
                env::AWS_ACCESS_KEY_ID,
                model.access_key_id
            ));
            lines.push(format!(
                "export {}=\"{}\"",
                env::AWS_SECRET_ACCESS_KEY,
                model.secret_access_key
            ));
            lines.push(format!(
                "export {}=\"{}\"",
                env::AWS_SESSION_TOKEN,
                model.session_token
            ));
            lines.push(format!(
                "export {}=\"{}\"",
                env::AWS_SECURITY_TOKEN,
                model.session_token
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
            lines.push(format!("# eval $({})", args));
            lines.join("\n")
        }
        CredentialOutputTarget::Fish => {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!(
                "set -gx {} \"{}\"",
                env::AWS_ACCESS_KEY_ID,
                model.access_key_id
            ));
            lines.push(format!(
                "set -gx {} \"{}\"",
                env::AWS_SECRET_ACCESS_KEY,
                model.secret_access_key
            ));
            lines.push(format!(
                "set -gx {} \"{}\"",
                env::AWS_SESSION_TOKEN,
                model.session_token
            ));
            lines.push(format!(
                "set -gx {} \"{}\"",
                env::AWS_SECURITY_TOKEN,
                model.session_token
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
            lines.push(format!("# {} | source", args));

            lines.join("\n")
        }
        CredentialOutputTarget::PowerShell => {
            let mut lines: Vec<String> = Vec::new();

            lines.push(format!(
                "$env:{}=\"{}\"",
                env::AWS_ACCESS_KEY_ID,
                model.access_key_id
            ));
            lines.push(format!(
                "$env:{}=\"{}\"",
                env::AWS_SECRET_ACCESS_KEY,
                model.secret_access_key
            ));
            lines.push(format!(
                "$env:{}=\"{}\"",
                env::AWS_SESSION_TOKEN,
                model.session_token
            ));
            lines.push(format!(
                "$env:{}=\"{}\"",
                env::AWS_SECURITY_TOKEN,
                model.session_token
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
            lines.push(format!("# {} | Invoke-Expression", args));

            lines.join("\n")
        }
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
                model.access_key_id,
            );
            profile.insert(
                shared_credentials::AWS_SECRET_ACCESS_KEY.to_string(),
                model.secret_access_key,
            );
            profile.insert(
                shared_credentials::AWS_SESSION_TOKEN.to_string(),
                model.session_token.clone(),
            );
            profile.insert(
                shared_credentials::AWS_SECURITY_TOKEN.to_string(),
                model.session_token,
            );

            {
                let expires = credential.expiration();
                let datetime = chrono::DateTime::from_timestamp(expires.secs(), expires.subsec_nanos())
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
