use crate::base::{Cmd, Validation};
use crate::models::configuration::v0::MasqueradeConfig as ConfigV0;
use crate::models::configuration::v1::Configuration as ConfigV1;
use crate::variables::cmd::target;
use clap::{ArgMatches, Command};

pub struct List;

impl Cmd for List {
    const NAME: &'static str = target::sub_command::LIST;

    fn subcommand() -> Command {
        Command::new(Self::NAME).about("list target name")
    }

    fn run(_args: &ArgMatches) -> Result<(), String> {
        let (path, version) = crate::path::get_current_path_masquerade_config()?;
        let text = crate::fs::load_text(&path)?;
        let config = match version {
            crate::variables::models::configuration::Version::V0 => ConfigV0::new(&text)?.migrate(),
            crate::variables::models::configuration::Version::V1 => ConfigV1::new(&text)?,
        };

        config.validate()?;

        let result: Vec<&String> = config.target.iter().map(|t| &t.name).collect();
        let text = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("failed to serialize result: {}", e))?;

        println!("{}", text);
        Ok(())
    }
}
