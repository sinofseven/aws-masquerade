use crate::lib::cmd_base::Cmd;
use crate::lib::fs::load_config;
use clap::{App, ArgMatches, SubCommand};

pub const NAME: &str = "validate";
pub struct Validate;

impl Cmd for Validate {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("validate config")
    }

    fn run(_: &ArgMatches) -> Result<(), String> {
        load_config().map(|_| ())
    }
}
