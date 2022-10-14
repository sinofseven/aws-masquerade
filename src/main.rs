pub mod base;
pub mod cmd;
pub mod fs;
pub mod io;
pub mod models;
pub mod path;
pub mod totp;
pub mod variables;

use base::Cmd;
use cmd::{Assume, Configure, Source, Target};

use clap::command;

fn main() -> Result<(), String> {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Configure::subcommand())
        .subcommand(Source::subcommand())
        .subcommand(Target::subcommand())
        .subcommand(Assume::subcommand())
        .get_matches();
    match matches.subcommand() {
        Some((Configure::NAME, args)) => Configure::run(args),
        Some((Source::NAME, args)) => Source::run(args),
        Some((Target::NAME, args)) => Target::run(args),
        Some((Assume::NAME, args)) => Assume::run(args),
        _ => unreachable!(""),
    }
}
