use clap::{ArgMatches, Command};
use crate::base::Cmd;
use crate::variables::cmd::source;

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
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
    }
}

impl Cmd for Show {
    const NAME: &'static str = source::sub_command::SHOW;

    fn subcommand() -> Command {
        Command::new(Self::NAME)
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        todo!()
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