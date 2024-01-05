use clap::{ArgMatches, Command};

pub trait Cmd {
    const NAME: &'static str;
    fn subcommand() -> Command;
    fn run(args: &ArgMatches) -> Result<(), String>;
}
