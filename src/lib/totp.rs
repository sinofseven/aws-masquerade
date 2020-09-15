// I wrote with reference to the following code of MIT license.
//   https://github.com/evansmurithi/cloak/blob/2318bbdd45/src/otp.rs
//   license: https://github.com/evansmurithi/cloak/blob/2318bbdd45/LICENSE

use data_encoding::BASE32_NOPAD;
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct TOTP {
    decoded_key: Vec<u8>,
    digits: usize,
    output_base: Vec<u8>,
}

impl TOTP {
    pub fn new(key: &String) -> Result<TOTP, String> {
        let decoded_key = BASE32_NOPAD
            .decode(key.as_bytes())
            .map_err(|err| format!("failed to decode totp secret: {}", err))?;
        Ok(TOTP {
            decoded_key: decoded_key,
            digits: 6,
            output_base: "0123456789".to_owned().into_bytes(),
        })
    }

    fn get_counter(&self) -> u64 {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        time / 30
    }

    fn encode_digest(&self, digest: &[u8]) -> String {
        let offset = (*digest.last().unwrap() & 0xf) as usize;
        let snum = ((u32::from(digest[offset]) & 0x7f) << 24)
            | ((u32::from(digest[offset + 1]) & 0xff) << 16)
            | ((u32::from(digest[offset + 2]) & 0xff) << 8)
            | (u32::from(digest[offset + 3]) & 0xff);
        let base = self.output_base.len() as u32;
        let hotp_code = snum % base.pow(self.digits as u32);
        let code = format!("{:0width$}", hotp_code, width = self.digits);
        code
    }

    pub fn generate(&self) -> String {
        let counter = self.get_counter();
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
        let signing_key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &self.decoded_key);
        let digest = hmac::sign(&signing_key, &message);
        self.encode_digest(digest.as_ref())
    }
}
