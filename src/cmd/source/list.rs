use crate::base::{Cmd, Validation};
use crate::variables::cmd::source;
use clap::{ArgMatches, Command};

pub struct List;

impl Cmd for List {
    const NAME: &'static str = source::sub_command::LIST;

    fn subcommand() -> Command {
        Command::new(Self::NAME).about("list source name")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let config = crate::models::configuration::load_configuration()?;

        config.validate()?;

        let result: Vec<&String> = config.source.iter().map(|s| &s.name).collect();
        let text = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("failed to serialize result: {}", e))?;

        println!("{}", text);

        Ok(())
    }
}
