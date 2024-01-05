use crate::base::Cmd;
use crate::variables::cmd::source;
use clap::{arg, ArgMatches, Command};

pub struct Show;

impl Cmd for Show {
    const NAME: &'static str = source::sub_command::SHOW;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("show detail of a source")
            .arg(arg!(<SOURCE_NAME>))
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let source_name: &String = args.get_one("SOURCE_NAME").unwrap();
        let config = crate::models::configuration::load_configuration()?;

        let source = config
            .source
            .iter()
            .find(|s| &s.name == source_name)
            .ok_or_else(|| format!("source(name={}) is not found.", source_name))?;

        let text = serde_json::to_string_pretty(source)
            .map_err(|e| format!("failed to serialize source: {}", e))?;

        println!("{}", text);

        Ok(())
    }
}
