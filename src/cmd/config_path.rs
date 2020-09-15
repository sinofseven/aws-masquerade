use crate::lib::cmd_base::Cmd;
use crate::lib::dirs::MASQUERADE_PATH;
use clap::{App, ArgMatches, SubCommand};

pub const NAME: &str = "config-path";
pub struct ConfigPath;

impl Cmd for ConfigPath {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("show path of config file")
    }

    fn run(_: &ArgMatches) -> Result<(), String> {
        let path = MASQUERADE_PATH.config();
        println!("{}", path.to_str().unwrap());
        Ok(())
    }
}
