use clap::{App, ArgMatches};

pub trait Cmd {
    fn subcommand<'a, 'b>() -> App<'a, 'b>;
    fn run(args: &ArgMatches) -> Result<(), String>;
}
