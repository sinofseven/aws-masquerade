use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{load_config, save_config};
use clap::{App, Arg, ArgMatches, SubCommand};

pub const NAME: &str = "remove";
pub struct Remove;

impl Cmd for Remove {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("remove a account").arg(
            Arg::with_name("account")
                .required(true)
                .long("account-name")
                .short("a")
                .takes_value(true)
                .help("Name of the account"),
        )
    }

    fn run(args: &ArgMatches) -> Result<(), String> {
        let account_name = args.value_of("account").unwrap();
        let mut config = load_config()?;
        let data = match config.accounts.get(account_name) {
            None => return Err(format!("Account \"{}\" does not exist.", account_name)),
            Some(data) => data,
        };

        return if crate::cmd::add::confirm(
            &account_name.to_string(),
            data,
            "Delete Account",
            "Do you confirm delete account? (y/n) [n]: ",
            false,
        ) {
            config.accounts.remove(account_name);
            save_config(&config)
        } else {
            Ok(())
        };
    }
}
