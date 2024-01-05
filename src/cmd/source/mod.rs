mod list;
mod show;

use crate::base::Cmd;
use crate::variables::cmd::source;
use clap::{ArgMatches, Command};

use list::List;
use show::Show;

pub struct Source;

impl Cmd for Source {
    const NAME: &'static str = source::NAME;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
            .about("Commands related to source configuration")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(List::subcommand())
            .subcommand(Show::subcommand())
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        match args.subcommand() {
            Some((List::NAME, sub_args)) => List::run(sub_args),
            Some((Show::NAME, sub_args)) => Show::run(sub_args),
            _ => unreachable!("This is Bug in 'cmd/source.rs'."),
        }
    }
}
