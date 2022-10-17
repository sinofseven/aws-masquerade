use colored::Colorize;
use data_encoding::BASE32_NOPAD;
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_counter(key: &str, is_save_totp_counter_history: &bool) -> Result<u64, String> {
    let unixtime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("failed to calculate unixtime: {}", e))?;
    let counter = unixtime / 30;
    let path = crate::path::get_path_totp_generate_history()?;

    if !is_save_totp_counter_history || !path.exists() {
        return Ok(counter);
    }

    let text = crate::fs::load_text(&path)
        .map_err(|e| format!("failed to read last totp counter: {}", e))?;
    let map_last_counter: std::collections::HashMap<String, u64> = serde_json::from_str(&text)
        .map_err(|e| format!("failed to deserialize totp last counter: {}", e))?;

    let next_counter = match map_last_counter.get(key) {
        Some(last_counter) => last_counter + 1,
        None => return Ok(counter),
    };

    let next_unixtime = next_counter * 30;
    if next_unixtime < unixtime {
        return Ok(counter);
    }

    let duration = std::time::Duration::from_secs(next_unixtime - unixtime);
    eprintln!(
        "{}",
        format!(
            "this mfa secret is already used in this counter. so wait {:?}",
            duration
        )
        .yellow()
    );
    std::thread::sleep(duration);

    Ok(next_counter)
}

fn save_totp_counter_history(key: &str, counter: &u64) -> Result<(), String> {
    let path = crate::path::get_path_totp_generate_history()?;
    let mut data: std::collections::HashMap<String, u64> = if path.exists() {
        let text = crate::fs::load_text(&path)?;
        serde_json::from_str(&text)
            .map_err(|e| format!("failed to deserialize totp last counter: {}", e))?
    } else {
        std::collections::HashMap::new()
    };

    data.insert(key.to_string(), *counter);

    let text = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("failed to seralize totp last counter: {}", e))?;
    crate::fs::save_text(&path, &text)
}

fn encode_digest(digest: &[u8]) -> String {
    let digits = 6_usize;
    let offset = (*digest.last().unwrap() & 0xf) as usize;
    let snum = ((u32::from(digest[offset]) & 0x7f) << 24)
        | ((u32::from(digest[offset + 1]) & 0xff) << 16)
        | ((u32::from(digest[offset + 2]) & 0xff) << 8)
        | (u32::from(digest[offset + 3]) & 0xff);
    let output_chars = "0123456789".to_owned().into_bytes();
    let base = output_chars.len() as u32;
    let hotp_code = snum % base.pow(digits as u32);
    format!("{:0width$}", hotp_code, width = digits)
}

pub fn generate(secret: &String, is_save_totp_counter_history: &bool) -> Result<String, String> {
    let key = BASE32_NOPAD
        .decode(secret.as_bytes())
        .map_err(|e| format!("failed to decode totp secred: {e}"))?;
    let counter = get_counter(secret, is_save_totp_counter_history)?;
    let message: [u8; 8] = [
        ((counter >> 56) & 0xff) as u8,
        ((counter >> 48) & 0xff) as u8,
        ((counter >> 40) & 0xff) as u8,
        ((counter >> 32) & 0xff) as u8,
        ((counter >> 24) & 0xff) as u8,
        ((counter >> 16) & 0xff) as u8,
        ((counter >> 8) & 0xff) as u8,
        (counter & 0xff) as u8,
    ];
    let signed_key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &key);
    let digest = hmac::sign(&signed_key, &message);

    if *is_save_totp_counter_history {
        save_totp_counter_history(secret, &counter)?;
    }

    Ok(encode_digest(digest.as_ref()))
}
