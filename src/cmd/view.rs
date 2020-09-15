use crate::lib::cmd_base::Cmd;
use crate::lib::fs::{load_config, Account};
use clap::{App, Arg, ArgMatches, SubCommand};
use std::collections::HashMap;

pub const NAME: &str = "view";
pub struct View;

impl Cmd for View {
    fn subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name(NAME).about("view a account").arg(
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
        let config = load_config()?;
        let account_data = match config.accounts.get(account_name) {
            None => return Err(format!("Account \"{}\" does not exist.", account_name)),
            Some(data) => data,
        };
        let mut map: HashMap<&str, &Account> = HashMap::new();
        map.insert(account_name, account_data);
        let text = serde_json::to_string_pretty(&map).unwrap();
        println!("{}", text);

        Ok(())
    }
}
