use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::anyhow;
use base64::Engine as _;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PREFIX: &'static str = "ENCRYPTED:";

#[derive(Serialize, Deserialize)]
struct Encrypted {
    nonce: Vec<u8>,
    payload: Vec<u8>,
}

fn secret() -> String {
    option_env!("OORT_CODE_ENCRYPTION_SECRET")
        .unwrap_or("not a secret")
        .to_string()
}

fn aes_key() -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&secret());
    hasher.finalize().to_vec()
}

pub fn is_encrypted(text: &str) -> bool {
    text.starts_with(PREFIX)
}

fn random_nonce() -> Vec<u8> {
    let mut data = [0u8; 12];
    rand::thread_rng().fill(&mut data);
    data.to_vec()
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new_from_slice(&aes_key())?;
    let nonce_data = random_nonce();
    let nonce = Nonce::from_slice(nonce_data.as_slice());
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    let serialized = bincode::serialize(&Encrypted {
        nonce: nonce_data,
        payload: ciphertext,
    })?;
    let base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(serialized);
    Ok(format!("{}{}", PREFIX, base64))
}

pub fn decrypt(mut input: &str) -> anyhow::Result<String> {
    input = input.trim();
    input = match input.strip_prefix(PREFIX) {
        Some(x) => x,
        None => return Err(anyhow!("missing encryption prefix")),
    };
    let serialized: Vec<u8> = base64::engine::general_purpose::STANDARD_NO_PAD.decode(input)?;
    let deserialized: Encrypted = bincode::deserialize(&serialized[..])?;
    let cipher = Aes256Gcm::new_from_slice(&aes_key())?;
    let nonce = Nonce::from_slice(&deserialized.nonce[..]);
    let plaintext = cipher.decrypt(nonce, &deserialized.payload[..])?;
    Ok(std::str::from_utf8(&plaintext)?.to_owned())
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_basic() -> anyhow::Result<()> {
        let plaintext = "Hello world!";
        let encrypted = encrypt(plaintext)?;
        assert!(encrypted.starts_with(PREFIX));
        let decrypted = decrypt(&encrypted)?;
        assert_eq!(decrypted, plaintext);
        Ok(())
    }
}
