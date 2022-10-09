use crate::base::{Cmd, Validation};
use crate::variables::cmd::source;
use clap::{arg, ArgMatches, Command};

pub struct Source;
struct List;
struct Show;
struct Add;
struct Edit;
struct Remove;

impl Cmd for Source {
    const NAME: &'static str = source::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("Commands related to source configuration")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(List::subcommand())
            .subcommand(Show::subcommand())
            .subcommand(Add::subcommand())
            .subcommand(Edit::subcommand())
            .subcommand(Remove::subcommand())
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        match args.subcommand() {
            Some((List::NAME, sub_args)) => List::run(sub_args),
            Some((Show::NAME, sub_args)) => Show::run(sub_args),
            Some((Add::NAME, sub_args)) => Add::run(sub_args),
            Some((Edit::NAME, sub_args)) => Edit::run(sub_args),
            Some((Remove::NAME, sub_args)) => Remove::run(sub_args),
            _ => unreachable!("This is Bug in 'cmd/source.rs'."),
        }
    }
}

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

impl Cmd for Show {
    const NAME: &'static str = source::sub_command::SHOW;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("show source detail")
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

impl Cmd for Add {
    const NAME: &'static str = source::sub_command::ADD;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}

impl Cmd for Edit {
    const NAME: &'static str = source::sub_command::EDIT;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}

impl Cmd for Remove {
    const NAME: &'static str = source::sub_command::REMOVE;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}
