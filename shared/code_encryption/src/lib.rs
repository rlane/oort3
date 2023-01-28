use age::armor::{ArmoredReader, ArmoredWriter};
use age::secrecy::Secret;
use std::io::{Read, Write};

fn secret() -> String {
    option_env!("OORT_CODE_ENCRYPTION_SECRET")
        .unwrap_or("not a secret")
        .to_string()
}

pub fn is_encrypted(text: &str) -> bool {
    text.starts_with("-----BEGIN AGE ENCRYPTED FILE-----")
}

pub fn encrypt(plaintext: &str) -> anyhow::Result<String> {
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(secret()));
    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(ArmoredWriter::wrap_output(
        &mut encrypted,
        age::armor::Format::AsciiArmor,
    )?)?;
    writer.write_all(plaintext.as_bytes())?;
    writer.finish().and_then(|armor| armor.finish())?;
    Ok(std::str::from_utf8(&encrypted)?.to_string())
}

pub fn decrypt(ciphertext: &str) -> anyhow::Result<String> {
    let decryptor = match age::Decryptor::new(ArmoredReader::new(ciphertext.as_bytes()))? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(anyhow::anyhow!("failed to create decryptor")),
    };
    let mut decrypted = vec![];
    let mut reader = decryptor.decrypt(&Secret::new(secret()), None)?;
    reader.read_to_end(&mut decrypted)?;
    Ok(std::str::from_utf8(&decrypted)?.to_string())
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_basic() -> anyhow::Result<()> {
        let plaintext = "Hello world!";
        let encrypted = encrypt(plaintext)?;
        let decrypted = decrypt(&encrypted)?;
        assert_eq!(decrypted, plaintext);
        Ok(())
    }
}
