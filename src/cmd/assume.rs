use crate::base::{Cmd, Validation};
use crate::models::configuration::v1;
use crate::models::configuration::v1::{CliOutputTarget, CredentialOutputTarget};
use crate::variables::cmd::assume;
use crate::variables::output::environment_variables as env;
use crate::variables::output::shared_credentials;
use aws_types::Credentials;
use clap::{arg, ArgMatches, Command};
use serde::Serialize;
use std::collections::BTreeMap;

pub struct Assume;

impl Cmd for Assume {
    const NAME: &'static str = assume::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("execute assume role")
            .arg(arg!(<TARGET_NAME>))
            .arg(
                arg!(-c <CREDENTIAL_OUTPUT> "output of assume role result")
                    .long("credential-output")
                    .value_parser(clap::builder::EnumValueParser::<CredentialOutputTarget>::new()),
            )
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let name_target: &String = args.get_one("TARGET_NAME").unwrap();

        let config = crate::models::configuration::load_configuration()?;
        config.validate()?;
        let is_save_totp_last_counter = &config
            .core
            .save_totp_counter_history
            .map_or_else(|| false, |f| f);

        let target = config
            .target
            .iter()
            .find(|t| &t.name == name_target)
            .ok_or_else(|| format!("target(name={}) is not found.", name_target))?;
        let source = config
            .source
            .iter()
            .find(|s| s.name == target.source)
            .ok_or_else(|| format!("source(name={} is not found.", &target.source))?;

        let credential_output = match args.get_one::<CredentialOutputTarget>("CREDENTIAL_OUTPUT") {
            Some(output) => output,
            None => &target.credential_output,
        };

        let resp = tokio::runtime::Runtime::new()
            .map_err(|e| format!("failed to create async runtime: {}", e))?
            .block_on(exec_assume(source, target, is_save_totp_last_counter))
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
    let mut config_loader = aws_config::from_env();
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
) -> Result<aws_sdk_sts::output::AssumeRoleOutput, String> {
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
        .map_err(|e| format!("failed to assume role: {}", e))
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
    output_assume_role: &aws_sdk_sts::output::AssumeRoleOutput,
) -> Result<(), String> {
    let args = {
        let args: Vec<String> = std::env::args().collect();
        args.join(" ")
    };
    let credential = output_assume_role
        .credentials()
        .ok_or_else(|| "there is not credentials in assume role result.".to_string())?;
    let model = JsonCredential {
        access_key_id: credential
            .access_key_id()
            .ok_or_else(|| {
                format!(
                    "there is not access_key_id in credentials. {:?}",
                    credential
                )
            })?
            .to_string(),
        secret_access_key: credential
            .secret_access_key()
            .ok_or_else(|| {
                format!(
                    "there is not secret_access_key in credentials. {:?}",
                    credential
                )
            })?
            .to_string(),
        session_token: credential
            .session_token()
            .ok_or_else(|| format!("there is not session_token. {:?}", credential))?
            .to_string(),
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
            let text = crate::fs::load_text(&path)?;
            let mut configure: BTreeMap<String, BTreeMap<String, String>> =
                serde_ini::from_str(&text).map_err(|e| {
                    format!("failed to deserialize aws shared credential file: {}", e)
                })?;

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

            if let Some(expires) = credential.expiration() {
                let naive =
                    chrono::NaiveDateTime::from_timestamp(expires.secs(), expires.subsec_nanos());
                let datetime: chrono::DateTime<chrono::Utc> =
                    chrono::DateTime::from_utc(naive, chrono::Utc);
                profile.insert(
                    shared_credentials::X_SECURITY_TOKEN_EXPIRES.to_string(),
                    datetime.to_rfc3339(),
                );
            }

            if let Some(assumed_role_user) = output_assume_role.assumed_role_user() {
                if let Some(arn) = assumed_role_user.arn() {
                    profile.insert(
                        shared_credentials::X_PRINCIPAL_ARN.to_string(),
                        arn.to_string(),
                    );
                }
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
