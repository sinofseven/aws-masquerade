use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{
    load_config, save_config, Account, AwsCliOutput, CredentialOutputTarget, MasqueradeConfig,
};
use crate::lib::io::{get_confirm_with_default, get_input};
use crate::lib::totp::TOTP;
use clap::{App, ArgMatches, SubCommand};
use regex::Regex;
use rusoto_core::Region;
use std::collections::HashMap;
use std::str::FromStr;

pub struct Add;
pub const NAME: &str = "add";

impl Cmd for Add {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("add a account")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let mut config = match load_config() {
            Ok(config) => config,
            Err(_) => MasqueradeConfig::new(),
        };

        let mut data = Account {
            source_profile: None,
            role_arn: "".to_string(),
            mfa_arn: None,
            mfa_secret: None,
            credential_output: CredentialOutputTarget::SharedCredentials,
            output: None,
            region: None,
        };

        let mut name = "".to_string();
        while {
            let (input_name, input_data) = input_account_data(&config, &name, &data, true);
            data = input_data;
            name = input_name;

            !confirm(
                &name,
                &data,
                "\nGenerated Account\n",
                "\nDo you confirm add account? (y/n) [y]: ",
                true,
            )
        } {}
        config.accounts.insert(name, data);

        save_config(&config)
    }
}

pub fn input_account_data(
    config: &MasqueradeConfig,
    old_name: &String,
    old_data: &Account,
    is_create: bool,
) -> (String, Account) {
    let account_name = if is_create {
        input_account_name(&config, old_name)
    } else {
        old_name.clone()
    };
    let source_profile = input_source_profile_name(&old_data.source_profile);
    let role_arn = input_role_arn(&old_data.role_arn);
    let mfa_arn = input_mfa_arn(&old_data.mfa_arn);
    let mfa_secret = match mfa_arn {
        None => None,
        Some(_) => input_mfa_secret(&old_data.mfa_secret),
    };
    let credential_output = input_credential_output(&old_data.credential_output);
    let cli_output = input_cli_output(&old_data.output);
    let default_region = input_default_region(&old_data.region);

    let account_data = Account {
        source_profile: source_profile,
        role_arn: role_arn,
        mfa_arn: mfa_arn,
        mfa_secret: mfa_secret,
        credential_output: credential_output,
        output: cli_output,
        region: default_region,
    };

    (account_name, account_data)
}

fn input_account_name(config: &MasqueradeConfig, old_name: &String) -> String {
    loop {
        let suffix = if old_name.is_empty() {
            "".to_string()
        } else {
            format!(" [{}]", old_name)
        };
        let mut name = get_input(format!("account name (required){}: ", suffix));
        if name.is_empty() {
            if old_name.is_empty() {
                println!("   account name is required.");
                continue;
            } else {
                name = old_name.clone();
            }
        }
        if config.accounts.contains_key(&name) {
            println!("   account name is existed.");
            continue;
        }
        return name;
    }
}

fn input_source_profile_name(old_source_profile: &Option<String>) -> Option<String> {
    let suffix = if let Some(old) = old_source_profile {
        old.clone()
    } else {
        "".to_string()
    };
    loop {
        let name = get_input(format!("source profile name [{}]: ", suffix));
        return if name.is_empty() {
            if let Some(old) = old_source_profile {
                match get_confirm_with_default(
                    format!("Do you remove \"{}\"? (y/n) [n]: ", old),
                    false,
                ) {
                    Err(_) => {
                        println!("   invalid input");
                        continue;
                    }
                    Ok(is_remove) => {
                        if is_remove {
                            None
                        } else {
                            Some(old.clone())
                        }
                    }
                }
            } else {
                None
            }
        } else {
            Some(name)
        };
    }
}

fn input_role_arn(old_role_arn: &String) -> String {
    let re = Regex::new(r"^arn:aws:iam::\d+:role/([\u0021-\u007F]+/|)[\w+=,.@-]+$").unwrap();
    loop {
        let suffix = if old_role_arn.is_empty() {
            "".to_string()
        } else {
            format!(" [{}]", old_role_arn)
        };
        let mut arn = get_input(format!("role arn (required){}: ", suffix));
        if arn.is_empty() {
            if old_role_arn.is_empty() {
                println!("   role arn is required.");
                continue;
            } else {
                arn = old_role_arn.clone();
            }
        }
        if !re.is_match(&arn) {
            println!("   invalid role arn!!");
            continue;
        }
        return arn;
    }
}

fn input_mfa_arn(old_mfa_arn: &Option<String>) -> Option<String> {
    let default = if let Some(mfa) = old_mfa_arn {
        mfa.clone()
    } else {
        "".to_string()
    };
    loop {
        let arn = get_input(format!("mfa arn [{}]: ", default));
        if arn.is_empty() {
            if default.is_empty() {
                return None;
            } else {
                match get_confirm_with_default(
                    format!("Do you remove \"{}\"? (y/n) [n]: ", default),
                    false,
                ) {
                    Err(_) => {
                        println!("   invalid input");
                        continue;
                    }
                    Ok(is_remove) => return if is_remove { None } else { Some(default) },
                }
            }
        } else {
            return Some(arn);
        }
    }
}

fn input_mfa_secret(old_secret: &Option<String>) -> Option<String> {
    let default = if let Some(secret) = old_secret {
        secret.clone()
    } else {
        "".to_string()
    };
    loop {
        let mut secret = get_input(format!("mfa secret [{}]: ", default));
        if secret.is_empty() {
            if default.is_empty() {
                return None;
            } else {
                match get_confirm_with_default(
                    format!("Do you remove \"{}\"? (y/n) [n]", default),
                    false,
                ) {
                    Err(_) => {
                        println!("   invalid input");
                        continue;
                    }
                    Ok(is_remove) => {
                        if is_remove {
                            return None;
                        } else {
                            secret = default.clone()
                        }
                    }
                }
            }
        }
        match TOTP::new(&secret) {
            Ok(_) => return Some(secret),
            Err(e) => println!("  invalid secret: {:?}", e),
        };
    }
}

fn input_credential_output(old_output: &CredentialOutputTarget) -> CredentialOutputTarget {
    loop {
        let default = match old_output {
            CredentialOutputTarget::SharedCredentials => "0",
            CredentialOutputTarget::Bash => "1",
            CredentialOutputTarget::Fish => "2",
            CredentialOutputTarget::PowerShell => "3",
        };

        println!("\nSelect Credential Output Type:");
        println!(" [0] {}", CredentialOutputTarget::SharedCredentials);
        println!(" [1] {}", CredentialOutputTarget::Bash);
        println!(" [2] {}", CredentialOutputTarget::Fish);
        println!(" [3] {}", CredentialOutputTarget::PowerShell);

        let number = get_input(format!("\n > [{}]: ", default));

        if number.is_empty() {
            return old_output.clone();
        }

        match number.as_str() {
            "0" => return CredentialOutputTarget::SharedCredentials,
            "1" => return CredentialOutputTarget::Bash,
            "2" => return CredentialOutputTarget::Fish,
            "3" => return CredentialOutputTarget::PowerShell,
            _ => println!("   Invalid Input"),
        }
    }
}

fn input_cli_output(old_output: &Option<AwsCliOutput>) -> Option<AwsCliOutput> {
    loop {
        let default = if let Some(o) = old_output {
            match o {
                AwsCliOutput::Json => "0",
                AwsCliOutput::Text => "1",
                AwsCliOutput::Table => "2",
            }
        } else {
            ""
        };
        println!("\nSelect awscli output type: ");
        println!(" [0] {}", AwsCliOutput::Json);
        println!(" [1] {}", AwsCliOutput::Text);
        println!(" [2] {}", AwsCliOutput::Table);

        let input = get_input(format!("\n > [{}]: ", default));
        if input.is_empty() {
            if let Some(o) = old_output {
                match get_confirm_with_default(
                    format!("Do you remove \"{}\"? (y/n) [n]: ", o),
                    true,
                ) {
                    Err(_) => {
                        println!("   invalid input");
                        continue;
                    }
                    Ok(is_remove) => return if is_remove { None } else { Some(o.clone()) },
                }
            } else {
                return None;
            }
        }
        match input.as_str() {
            "0" => return Some(AwsCliOutput::Json),
            "1" => return Some(AwsCliOutput::Text),
            "2" => return Some(AwsCliOutput::Table),
            _ => println!("   Invalid Input"),
        }
    }
}

fn input_default_region(old_region: &Option<Region>) -> Option<Region> {
    let default = if let Some(region) = old_region {
        region.name()
    } else {
        ""
    };
    loop {
        let region_name = get_input(format!("Default Region Name [{}]: ", default));
        if region_name.is_empty() {
            if let Some(old) = old_region {
                match get_confirm_with_default(
                    format!("Do you remove \"{}\"? (y/n) []: ", old.name()),
                    false,
                ) {
                    Err(_) => {
                        println!("   invalid input");
                        continue;
                    }
                    Ok(is_remove) => return if is_remove { None } else { Some(old.clone()) },
                }
            } else {
                return None;
            }
        }
        match Region::from_str(&region_name) {
            Ok(region) => return Some(region),
            Err(e) => println!("   parse region error: {}", e),
        }
    }
}

pub fn confirm<T>(
    account_name: &String,
    account_data: &Account,
    prefix_message: T,
    confirm_message: T,
    default: bool,
) -> bool
where
    T: std::fmt::Display + Copy,
{
    loop {
        println!("{}", prefix_message);

        let mut map: HashMap<String, Account> = HashMap::new();
        map.insert(account_name.clone(), account_data.clone());
        let json = serde_json::to_string_pretty(&map).unwrap();
        println!("{}", json);

        match get_confirm_with_default(confirm_message, default) {
            Err(_) => {
                println!("   invalid input");
                continue;
            }
            Ok(flag) => return flag,
        }
    }
}
