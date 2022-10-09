pub mod v0;
pub mod v1;

pub fn load_configuration() -> Result<v1::Configuration, String> {
    let (path, version) = crate::path::get_current_path_masquerade_config()?;
    let text = crate::fs::load_text(&path)?;

    match version {
        crate::variables::models::configuration::Version::V0 => Ok(v0::MasqueradeConfig::new(&text)?.migrate()),
        crate::variables::models::configuration::Version::V1 => v1::Configuration::new(&text),
    }
}
