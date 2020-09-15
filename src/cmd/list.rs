use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{load_config, MasqueradeConfig};
use clap::{App, ArgMatches, SubCommand};

pub const NAME: &str = "list";
pub struct List;

impl Cmd for List {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("list accounts")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let config = match load_config() {
            Ok(config) => config,
            Err(_) => MasqueradeConfig::new(),
        };
        for name in config.accounts.keys() {
            println!(" {}", name);
        }
        Ok(())
    }
}
