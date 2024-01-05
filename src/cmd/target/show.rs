use crate::base::{Cmd, Validation};
use crate::variables::cmd::target;
use clap::{arg, ArgMatches, Command};

pub struct Show;

impl Cmd for Show {
    const NAME: &'static str = target::sub_command::SHOW;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("show detail of a target")
            .arg(arg!(<TARGET_NAME>))
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let name_target: &String = args.get_one("TARGET_NAME").unwrap();

        let config = crate::models::configuration::load_configuration()?;
        config.validate()?;

        let target = config
            .target
            .iter()
            .find(|t| &t.name == name_target)
            .ok_or_else(|| format!("target(name={}) is not found.", name_target))?;

        let text = serde_json::to_string_pretty(&target)
            .map_err(|e| format!("failed to serialize target: {}", e))?;

        println!("{}", text);

        Ok(())
    }
}
