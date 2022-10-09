use crate::base::{Cmd, Validation};
use crate::models::configuration::v0::MasqueradeConfig as ConfigV0;
use crate::models::configuration::v1::Configuration as ConfigV1;
use crate::path;
use crate::variables::cmd::configure as names;
use crate::variables::models::configuration::Version as ConfigVersion;
use clap::{ArgMatches, Command};

pub struct Configure;
struct Path;
struct Validate;
struct Migrate;

impl Cmd for Configure {
    const NAME: &'static str = names::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("Commands related to configuration files")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(Path::subcommand())
            .subcommand(Validate::subcommand())
            .subcommand(Migrate::subcommand())
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        match args.subcommand() {
            Some((Path::NAME, sub_args)) => Path::run(sub_args),
            Some((Validate::NAME, sub_args)) => Validate::run(sub_args),
            Some((Migrate::NAME, sub_args)) => Migrate::run(sub_args),
            _ => unreachable!("This is Bug in 'cmd/configure.rs'."), // If all subcommands are defined above, anything else is unreachabe!()
        }
    }
}

impl Cmd for Path {
    const NAME: &'static str = names::sub_command::PATH;

    fn subcommand() -> Command {
        Command::new(Self::NAME).about("show config file path")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let (result, _) = path::get_current_path_masquerade_config()?;
        println!("{}", result.display());
        Ok(())
    }
}

impl Cmd for Validate {
    const NAME: &'static str = names::sub_command::VALIDATE;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let configure = crate::models::configuration::load_configuration()?;
        configure.validate()
    }
}

impl Cmd for Migrate {
    const NAME: &'static str = names::sub_command::MIGRATE;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let path_old = path::get_path_old_masquerade_config()?;
        let path_latest = path::get_path_masquerade_config()?;

        let text_old = crate::fs::load_text(&path_old)?;
        let config_old = crate::models::configuration::v0::MasqueradeConfig::new(&text_old)?;

        let config_latest = config_old.migrate();
        let text_latest = config_latest.to_string()?;

        crate::fs::save_text(&path_latest, &text_latest)
    }
}
