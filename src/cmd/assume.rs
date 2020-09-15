use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{
    add_into_shared_credentials, add_shared_config, load_config, Account, CredentialOutputTarget,
};
use crate::lib::io::{get_input, MasqueradeOutputExt};
use crate::lib::totp::TOTP;
use clap::{App, Arg, ArgMatches, SubCommand};
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_sts::{AssumeRoleRequest, AssumeRoleResponse, Sts, StsClient};

const TOKEN_ARG_NAME: &str = "token";

pub const NAME: &str = "assume";
pub struct Assume;

impl Cmd for Assume {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME)
            .about("exec assume role")
            .arg(
                Arg::with_name("account")
                    .required(true)
                    .long("account-name")
                    .short("a")
                    .takes_value(true)
                    .help("Name of the account"),
            )
            .arg(
                Arg::with_name(TOKEN_ARG_NAME)
                    .long("mfa-token")
                    .short("t")
                    .takes_value(true)
                    .help("Input Mfa Token"),
            )
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let account_name = args.value_of("account").unwrap();
        let config = load_config()?;
        let account_data = match config.accounts.get(account_name) {
            None => return Err(format!("Account \"{}\" does not exist.", account_name)),
            Some(data) => data,
        };

        let client = create_sts_client(account_data)?;
        let option = create_assume_role_option(args, account_data)?;
        let result = exec_assume_role(option, &client)?;

        output(&account_name.to_string(), account_data, &result)
    }
}

fn exec_assume_role(
    option: AssumeRoleRequest,
    client: &StsClient,
) -> Result<AssumeRoleResponse, String> {
    let mut runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => return Err(format!("failed to create async runtime: {}", e)),
    };
    let resp = runtime.block_on(client.assume_role(option));
    match resp {
        Ok(resp) => Ok(resp),
        Err(e) => Err(format!("failed to assume role: {}", e)),
    }
}

fn create_sts_client(account: &Account) -> Result<StsClient, String> {
    if let Some(source_profile) = &account.source_profile {
        let http_client = match HttpClient::new() {
            Ok(client) => client,
            Err(e) => return Err(format!("failed to create HTTP Client: {}", e)),
        };
        let mut provider = match ProfileProvider::new() {
            Ok(provider) => provider,
            Err(e) => {
                return Err(format!(
                    "failed to create Profile Credential Provider: {}",
                    e
                ))
            }
        };
        provider.set_profile(source_profile);
        Ok(StsClient::new_with(http_client, provider, Region::UsEast1))
    } else {
        Ok(StsClient::new(Region::UsEast1))
    }
}

fn create_assume_role_option(
    args: &ArgMatches,
    account: &Account,
) -> Result<AssumeRoleRequest, String> {
    let mut option = AssumeRoleRequest {
        role_arn: account.role_arn.clone(),
        role_session_name: uuid::Uuid::new_v4().to_string(),
        ..Default::default()
    };
    if let Some(mfa_arn) = &account.mfa_arn {
        option.serial_number = Some(mfa_arn.clone());
        option.token_code = Some(get_mfa_token(args, account)?);
    }

    Ok(option)
}

fn get_mfa_token(args: &ArgMatches, account: &Account) -> Result<String, String> {
    match args.value_of(TOKEN_ARG_NAME) {
        Some(token) => Ok(token.to_string()),
        None => {
            if let Some(secret) = &account.mfa_secret {
                let totp = TOTP::new(secret)?;
                Ok(totp.generate())
            } else {
                Ok(get_input("\nMFA TOKEN: "))
            }
        }
    }
}

fn output(
    account_name: &String,
    account_data: &Account,
    assume_result: &AssumeRoleResponse,
) -> Result<(), String> {
    let text = match account_data.credential_output {
        CredentialOutputTarget::Bash => {
            assume_result.create_bash_credentials(&account_data.output, &account_data.region)
        }
        CredentialOutputTarget::Fish => {
            assume_result.create_fish_credentials(&account_data.output, &account_data.region)
        }
        CredentialOutputTarget::PowerShell => {
            assume_result.create_power_shell_credentials(&account_data.output, &account_data.region)
        }
        CredentialOutputTarget::SharedCredentials => {
            let cred = assume_result.create_shared_credentials();
            if let Some(config) = account_data.create_shared_config() {
                add_shared_config(account_name, &config)?;
            }
            add_into_shared_credentials(account_name, &cred)?;

            println!("Your new access key pair has been stored in the AWS configuration");
            println!("To use this credential, call the AWS CLI with the --profile option (e.g. aws sts get-caller-identity --profile {})", account_name);
            return Ok(());
        }
    };

    println!("{}", text);
    Ok(())
}
