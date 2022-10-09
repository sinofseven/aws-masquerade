use crate::base::Cmd;
use clap::{ArgMatches, Command};
use std::path::PathBuf;

pub struct Test;

impl Cmd for Test {
    const NAME: &'static str = "test";

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let config_path = PathBuf::from("/Users/yuta/space/private/aws-masquerade/tmp/sample.toml");
        let text = crate::fs::load_text(&config_path)?;
        let data: crate::models::configuration::v1::Configuration =
            toml::from_str(&text).map_err(|e| format!("failed to deserialize config: {}", e))?;
        println!("{:?}", data);

        let output = toml::to_string(&data).unwrap();
        println!("{}", output);
        Ok(())
    }
}
