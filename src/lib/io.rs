// https://github.com/remind101/assume-role/blob/ca1eab460f3267fb7dde8685b0db52a4ea72e35d/main.go
// https://stackoverflow.com/questions/28370126/how-can-i-test-stdin-and-stdout

use crate::lib::fs::AwsCliOutput;
use rusoto_core::Region;
use rusoto_sts::{AssumeRoleResponse, Credentials};
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::io::Write;

pub trait MasqueradeOutputExt {
    fn create_bash_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String;
    fn create_fish_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String;
    fn create_power_shell_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String;
    fn create_shared_credentials(&self) -> HashMap<String, String>;
}

impl MasqueradeOutputExt for Credentials {
    fn create_bash_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        let args: Vec<String> = std::env::args().collect();

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "export AWS_ACCESS_KEY_ID=\"{}\"",
            self.access_key_id
        ));
        lines.push(format!(
            "export AWS_SECRET_ACCESS_KEY=\"{}\"",
            self.secret_access_key
        ));
        lines.push(format!(
            "export AWS_SESSION_TOKEN=\"{}\"",
            self.session_token
        ));
        lines.push(format!(
            "export AWS_SECURITY_TOKEN=\"{}\"",
            self.session_token
        ));
        if let Some(output) = output_type {
            lines.push(format!(
                "export AWS_DEFAULT_OUTPUT=\"{}\"",
                output.to_string()
            ))
        }
        if let Some(region) = default_region {
            lines.push(format!("export AWS_DEFAULT_REGION=\"{}\"", region.name()))
        }
        lines.push("# Run this to configure your shell:".to_string());
        lines.push(format!("# eval $({})", args.join(" ")));

        lines.join("\n")
    }

    fn create_fish_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        let args: Vec<String> = std::env::args().collect();

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "set -gx AWS_ACCESS_KEY_ID \"{}\"",
            self.access_key_id
        ));
        lines.push(format!(
            "set -gx AWS_SECRET_ACCESS_KEY \"{}\"",
            self.secret_access_key
        ));
        lines.push(format!(
            "set -gx AWS_SESSION_TOKEN \"{}\"",
            self.session_token
        ));
        lines.push(format!(
            "set -gx AWS_SECURITY_TOKEN \"{}\"",
            self.session_token
        ));
        if let Some(output) = output_type {
            lines.push(format!(
                "set -gx AWS_DEFAULT_OUTPUT \"{}\"",
                output.to_string()
            ))
        }
        if let Some(region) = default_region {
            lines.push(format!("set -gx AWS_DEFAULT_REGION \"{}\"", region.name()))
        }
        lines.push("# Run this to configure your shell:".to_string());
        lines.push(format!("# eval ({})", args.join(" ")));

        lines.join("\n")
    }

    fn create_power_shell_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        let args: Vec<String> = std::env::args().collect();

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("$env:AWS_ACCESS_KEY_ID=\"{}\"", self.access_key_id));
        lines.push(format!(
            "$env:AWS_SECRET_ACCESS_KEY=\"{}\"",
            self.secret_access_key
        ));
        lines.push(format!("$env:AWS_SESSION_TOKEN=\"{}\"", self.session_token));
        lines.push(format!(
            "$env:AWS_SECURITY_TOKEN=\"{}\"",
            self.session_token
        ));
        if let Some(output) = output_type {
            lines.push(format!(
                "$env:AWS_DEFAULT_OUTPUT=\"{}\"",
                output.to_string()
            ))
        }
        if let Some(region) = default_region {
            lines.push(format!("$env:AWS_DEFAULT_REGION=\"{}\"", region.name()))
        }
        lines.push("# Run this to configure your shell:".to_string());
        lines.push(format!("# eval $({})", args.join(" ")));

        lines.join("\n")
    }

    fn create_shared_credentials(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(
            "aws_access_key_id".to_string(),
            (&self.access_key_id).clone(),
        );
        map.insert(
            "aws_secret_access_key".to_string(),
            (&self.secret_access_key).clone(),
        );
        map.insert(
            "aws_session_token".to_string(),
            (&self.session_token).clone(),
        );
        map.insert(
            "aws_security_token".to_string(),
            (&self.session_token).clone(),
        );
        map.insert(
            "x_security_token_expires".to_string(),
            (&self.expiration).clone(),
        );

        map
    }
}

impl MasqueradeOutputExt for AssumeRoleResponse {
    fn create_bash_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        self.credentials
            .as_ref()
            .unwrap()
            .create_bash_credentials(output_type, default_region)
    }

    fn create_fish_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        self.credentials
            .as_ref()
            .unwrap()
            .create_fish_credentials(output_type, default_region)
    }

    fn create_power_shell_credentials(
        &self,
        output_type: &Option<AwsCliOutput>,
        default_region: &Option<Region>,
    ) -> String {
        self.credentials
            .as_ref()
            .unwrap()
            .create_power_shell_credentials(output_type, default_region)
    }

    fn create_shared_credentials(&self) -> HashMap<String, String> {
        let mut map = self
            .credentials
            .as_ref()
            .unwrap()
            .create_shared_credentials();
        map.insert(
            "x_principal_arn".to_string(),
            (&self.assumed_role_user.as_ref().unwrap().arn).clone(),
        );

        map
    }
}

#[test]
fn test_credentials_create_bash_credentials_1() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let actual = cred.create_bash_credentials(&None, &None);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_bash_credentials_2() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);

    let actual = cred.create_bash_credentials(&output, &None);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_OUTPUT="json""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_bash_credentials_3() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_bash_credentials(&None, &region);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_REGION="ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_bash_credentials_4() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_bash_credentials(&output, &region);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_OUTPUT="json""#,
        r#"export AWS_DEFAULT_REGION="ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_fish_credentials_1() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let actual = cred.create_fish_credentials(&None, &None);

    let lines_expected = [
        r#"set -gx AWS_ACCESS_KEY_ID "Adventurer (Bell Cranel)""#,
        r#"set -gx AWS_SECRET_ACCESS_KEY "The Blade of a God (Hestia Knife)""#,
        r#"set -gx AWS_SESSION_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_SECURITY_TOKEN "The Weak (Supporter)""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval ({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_fish_credentials_2() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);

    let actual = cred.create_fish_credentials(&output, &None);

    let lines_expected = [
        r#"set -gx AWS_ACCESS_KEY_ID "Adventurer (Bell Cranel)""#,
        r#"set -gx AWS_SECRET_ACCESS_KEY "The Blade of a God (Hestia Knife)""#,
        r#"set -gx AWS_SESSION_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_SECURITY_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_DEFAULT_OUTPUT "json""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval ({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_fish_credentials_3() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_fish_credentials(&None, &region);

    let lines_expected = [
        r#"set -gx AWS_ACCESS_KEY_ID "Adventurer (Bell Cranel)""#,
        r#"set -gx AWS_SECRET_ACCESS_KEY "The Blade of a God (Hestia Knife)""#,
        r#"set -gx AWS_SESSION_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_SECURITY_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_DEFAULT_REGION "ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval ({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_fish_credentials_4() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_fish_credentials(&output, &region);

    let lines_expected = [
        r#"set -gx AWS_ACCESS_KEY_ID "Adventurer (Bell Cranel)""#,
        r#"set -gx AWS_SECRET_ACCESS_KEY "The Blade of a God (Hestia Knife)""#,
        r#"set -gx AWS_SESSION_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_SECURITY_TOKEN "The Weak (Supporter)""#,
        r#"set -gx AWS_DEFAULT_OUTPUT "json""#,
        r#"set -gx AWS_DEFAULT_REGION "ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval ({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_power_shell_credentials_1() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let actual = cred.create_power_shell_credentials(&None, &None);

    let lines_expected = [
        r#"$env:AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"$env:AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"$env:AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"$env:AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_power_shell_credentials_2() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);

    let actual = cred.create_bash_credentials(&output, &None);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_OUTPUT="json""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_power_shell_credentials_3() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_bash_credentials(&None, &region);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_REGION="ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

#[test]
fn test_credentials_create_power_shell_credentials_4() {
    let cred = Credentials {
        access_key_id: "Adventurer (Bell Cranel)".to_string(),
        expiration: "Monster Festival (Monsterphilia)".to_string(),
        secret_access_key: "The Blade of a God (Hestia Knife)".to_string(),
        session_token: "The Weak (Supporter)".to_string(),
    };
    let output = Some(AwsCliOutput::Json);
    let region = Some(Region::ApNortheast1);

    let actual = cred.create_bash_credentials(&output, &region);

    let lines_expected = [
        r#"export AWS_ACCESS_KEY_ID="Adventurer (Bell Cranel)""#,
        r#"export AWS_SECRET_ACCESS_KEY="The Blade of a God (Hestia Knife)""#,
        r#"export AWS_SESSION_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_SECURITY_TOKEN="The Weak (Supporter)""#,
        r#"export AWS_DEFAULT_OUTPUT="json""#,
        r#"export AWS_DEFAULT_REGION="ap-northeast-1""#,
        r#"# Run this to configure your shell:"#,
    ];

    let joined: String = lines_expected.join("\n");
    let args: Vec<String> = std::env::args().collect();
    let eval = format!("\n# eval $({})", args.join(" "));
    let expected: String = joined + eval.as_str();

    assert_eq!(actual, expected);
}

// https://magidropack.hatenablog.com/entry/2018/12/18/194442
pub fn get_input<T>(message: T) -> String
where
    T: Display,
{
    print!("{}", message);
    let mut input = String::new();
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).ok();

    input.trim().to_string()
}

pub fn get_confirm_with_default<T>(message: T, default: bool) -> Result<bool, ()>
where
    T: Display,
{
    let resp = get_input(message);
    match resp.to_lowercase().as_str() {
        "" => Ok(default),
        "y" => Ok(true),
        "yes" => Ok(true),
        "n" => Ok(false),
        "no" => Ok(false),
        _ => Err(()),
    }
}

pub fn get_confirm<T>(message: T) -> Result<bool, ()>
where
    T: Display,
{
    let resp = get_input(message);
    match resp.to_lowercase().as_str() {
        "y" => Ok(true),
        "yes" => Ok(true),
        "n" => Ok(false),
        "no" => Ok(false),
        _ => Err(()),
    }
}
