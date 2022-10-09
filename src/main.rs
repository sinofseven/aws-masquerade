pub mod base;
pub mod cmd;
pub mod fs;
pub mod io;
pub mod models;
pub mod path;
pub mod variables;

use cmd::Test;

use base::Cmd;
use cmd::Configure;

use clap::command;

fn main() -> Result<(), String> {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Test::subcommand())
        .subcommand(Configure::subcommand())
        .get_matches();
    match matches.subcommand() {
        Some((Test::NAME, sub_matches)) => Test::run(sub_matches),
        Some((Configure::NAME, sub_matches)) => Configure::run(sub_matches),
        _ => unreachable!(""),
    }
}
