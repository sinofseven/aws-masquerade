use crate::base::Validation;

#[derive(clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ShowArgs {
    target_name: String,
}

pub fn run(args: ShowArgs) -> Result<(), String> {
    let config = crate::models::configuration::load_configuration()?;
    config.validate()?;

    let target = config
        .target
        .iter()
        .find(|t| t.name == args.target_name)
        .ok_or_else(|| format!("target(name={}) is not found.", args.target_name))?;

    let text = serde_json::to_string_pretty(&target)
        .map_err(|e| format!("failed to serialize target: {}", e))?;

    println!("{}", text);

    Ok(())
}
