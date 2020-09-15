use crate::cmd::add::{confirm, input_account_data};
use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{load_config, save_config};
use clap::{App, Arg, ArgMatches, SubCommand};

pub const NAME: &str = "edit";
pub struct Edit;

impl Cmd for Edit {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("edit a account").arg(
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

        let name = account_name.to_string();
        let (_, input_data) = input_account_data(&config, &name, &data, false);

        return if confirm(
            &name,
            &input_data,
            "\nUpdated Account\n",
            "\nDo you confirm edit account? (y/n) [n]: ",
            false,
        ) {
            config.accounts.insert(name, input_data);
            save_config(&config)
        } else {
            Ok(())
        };
    }
}
