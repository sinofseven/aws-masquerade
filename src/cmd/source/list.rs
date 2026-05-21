use crate::base::Validation;

pub fn run() -> Result<(), String> {
    let config = crate::models::configuration::load_configuration()?;

    config.validate()?;

    let result: Vec<&String> = config.source.iter().map(|s| &s.name).collect();
    let text = serde_json::to_string_pretty(&result)
        .map_err(|e| format!("failed to serialize result: {}", e))?;

    println!("{}", text);

    Ok(())
}
