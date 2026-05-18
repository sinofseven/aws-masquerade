#[derive(clap::Args)]
pub struct ShowArgs {
    source_name: String,
}

pub fn run(args: ShowArgs) -> Result<(), String> {
    let config = crate::models::configuration::load_configuration()?;

    let source = config
        .source
        .iter()
        .find(|s| s.name == args.source_name)
        .ok_or_else(|| format!("source(name={}) is not found.", args.source_name))?;

    let text = serde_json::to_string_pretty(source)
        .map_err(|e| format!("failed to serialize source: {}", e))?;

    println!("{}", text);

    Ok(())
}
