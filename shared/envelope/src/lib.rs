use sha2::{Digest, Sha256};

const DIGEST_LENGTH: usize = 32;

fn secret() -> Vec<u8> {
    option_env!("OORT_ENVELOPE_SECRET")
        .unwrap_or("not a secret")
        .as_bytes()
        .to_vec()
}

fn perturb(data: &[u8]) -> Vec<u8> {
    data.iter().map(|x| !x).collect()
}

fn digest(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(&secret());
    hasher.finalize().to_vec()
}

pub fn add(data: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(&digest(data));
    result.extend_from_slice(&perturb(data));
    result
}

pub fn remove(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < DIGEST_LENGTH {
        return None;
    }
    let (header, data) = data.split_at(DIGEST_LENGTH);
    let data = perturb(data);
    if digest(&data) != header {
        return None;
    }
    Some(data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let data = [0x11, 0x22, 0x33, 0x44];
        let enveloped = add(&data);
        assert_eq!(enveloped.len(), data.len() + DIGEST_LENGTH);
        assert_eq!(remove(&enveloped), Some(data.to_vec()));
    }

    #[test]
    fn test_corrupted_digest() {
        let data = [0x11, 0x22, 0x33, 0x44];
        let mut enveloped = add(&data);
        enveloped[4] = !enveloped[4];
        assert_eq!(remove(&enveloped), None);
    }

    #[test]
    fn test_corrupted_payload() {
        let data = [0x11, 0x22, 0x33, 0x44];
        let mut enveloped = add(&data);
        enveloped[35] = !enveloped[35];
        assert_eq!(remove(&enveloped), None);
    }
}
