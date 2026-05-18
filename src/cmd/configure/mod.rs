use crate::base::Validation;
use crate::path;

#[derive(clap::Args)]
#[command(
    about = "Commands related to configuration files",
    arg_required_else_help = true
)]
pub struct ConfigureArgs {
    #[command(subcommand)]
    command: ConfigureCommand,
}

#[derive(clap::Subcommand)]
enum ConfigureCommand {
    /// show config file path
    Path,
    Validate,
    Migrate,
}

impl ConfigureArgs {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            ConfigureCommand::Path => run_path(),
            ConfigureCommand::Validate => run_validate(),
            ConfigureCommand::Migrate => run_migrate(),
        }
    }
}

fn run_path() -> Result<(), String> {
    let (result, _) = path::get_current_path_masquerade_config()?;
    println!("{}", result.display());
    Ok(())
}

fn run_validate() -> Result<(), String> {
    let configure = crate::models::configuration::load_configuration()?;
    configure.validate()
}

fn run_migrate() -> Result<(), String> {
    let path_old = path::get_path_old_masquerade_config()?;
    let path_latest = path::get_path_masquerade_config()?;

    let text_old = crate::fs::load_text(&path_old)?;
    let config_old = crate::models::configuration::v0::MasqueradeConfig::new(&text_old)?;

    let config_latest = config_old.migrate();
    let text_latest = config_latest.to_string()?;

    crate::fs::save_text(&path_latest, &text_latest)
}
