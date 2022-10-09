use crate::base::Cmd;
use crate::variables::cmd::assume;
use clap::{ArgMatches, Command};

pub struct Assume;

impl Cmd for Assume {
    const NAME: &'static str = assume::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("execute assume role")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}