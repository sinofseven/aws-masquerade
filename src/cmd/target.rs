use crate::base::{Cmd, Validation};
use crate::models::configuration::v0::MasqueradeConfig as ConfigV0;
use crate::models::configuration::v1::Configuration as ConfigV1;
use crate::variables::cmd::target;
use clap::{ArgMatches, Command, arg};

pub struct Target;
struct List;
struct Show;
struct Add;
struct Edit;
struct Remove;

impl Cmd for Target {
    const NAME: &'static str = target::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("commands related to target configuration")
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
            _ => unreachable!("This is Bug in 'cmd/target.rs'."),
        }
    }
}

impl Cmd for List {
    const NAME: &'static str = target::sub_command::LIST;

    fn subcommand() -> Command {
        Command::new(Self::NAME).about("list target name")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let (path, version) = crate::path::get_current_path_masquerade_config()?;
        let text = crate::fs::load_text(&path)?;
        let config = match version {
            crate::variables::models::configuration::Version::V0 => ConfigV0::new(&text)?.migrate(),
            crate::variables::models::configuration::Version::V1 => ConfigV1::new(&text)?,
        };

        config.validate()?;

        let result: Vec<&String> = config.target.iter().map(|t| &t.name).collect();
        let text = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("failed to serialize result: {}", e))?;

        println!("{}", text);
        Ok(())
    }
}

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

        let target = config.target.iter().find(|t| &t.name == name_target).ok_or_else(|| format!("target(name={}) is not found.", name_target))?;

        let text = serde_json::to_string_pretty(&target).map_err(|e| format!("failed to serialize target: {}", e))?;

        println!("{}", text);

        Ok(())
    }
}

impl Cmd for Add {
    const NAME: &'static str = target::sub_command::ADD;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}

impl Cmd for Edit {
    const NAME: &'static str = target::sub_command::EDIT;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}

impl Cmd for Remove {
    const NAME: &'static str = target::sub_command::REMOTE;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}
