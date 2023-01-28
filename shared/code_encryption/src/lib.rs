use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::anyhow;
use base64::Engine as _;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use rand::Rng as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

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

fn compress(input: &str) -> anyhow::Result<Vec<u8>> {
    let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
    e.write_all(input.as_bytes())?;
    Ok(e.finish()?)
}

fn decompress(input: &[u8]) -> anyhow::Result<String> {
    let mut deflater = DeflateDecoder::new(&input[..]);
    let mut s = String::new();
    deflater.read_to_string(&mut s)?;
    Ok(s)
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new_from_slice(&aes_key())?;
    let nonce_data = random_nonce();
    let nonce = Nonce::from_slice(nonce_data.as_slice());
    let compressed = compress(plaintext)?;
    let ciphertext = cipher.encrypt(nonce, &compressed[..])?;
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
    let compressed = cipher.decrypt(nonce, &deserialized.payload[..])?;
    decompress(&compressed)
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
