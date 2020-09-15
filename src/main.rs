#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::lib::cmd_base::Cmd;
use clap::App;

mod cmd;
mod lib;

fn main() {
    let matches = App::new(crate_name!())
        .author("sinofseven")
        .about(crate_description!())
        .version(crate_version!())
        .subcommand(cmd::add::Add::subcommand())
        .subcommand(cmd::assume::Assume::subcommand())
        .subcommand(cmd::list::List::subcommand())
        .subcommand(cmd::view::View::subcommand())
        .subcommand(cmd::edit::Edit::subcommand())
        .subcommand(cmd::remove::Remove::subcommand())
        .subcommand(cmd::validate::Validate::subcommand())
        .subcommand(cmd::config_path::ConfigPath::subcommand())
        .get_matches();

    let result = match matches.subcommand() {
        (cmd::add::NAME, Some(arg)) => cmd::add::Add::run(&arg),
        (cmd::assume::NAME, Some(arg)) => cmd::assume::Assume::run(&arg),
        (cmd::list::NAME, Some(args)) => cmd::list::List::run(&args),
        (cmd::view::NAME, Some(args)) => cmd::view::View::run(&args),
        (cmd::edit::NAME, Some(args)) => cmd::edit::Edit::run(&args),
        (cmd::remove::NAME, Some(args)) => cmd::remove::Remove::run(&args),
        (cmd::validate::NAME, Some(args)) => cmd::validate::Validate::run(&args),
        (cmd::config_path::NAME, Some(args)) => cmd::config_path::ConfigPath::run(&args),
        _ => Err("No subcommand chosen. Add --help | -h to view the subcommands.".to_string()),
    };
    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
