use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;

pub fn check(text: &str) -> Result<(), anyhow::Error> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"\b(unsafe|extern|crate)\b|\b(macro_rules|include|include_bytes|include_str)\b|([^']static\b|^static\b)"#).unwrap();
    }
    if RE.is_match(text) {
        Err(anyhow!("Code did not pass sanitizer"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe() {
        assert!(check("... unsafe ...").is_err());
        assert!(check("... }unsafe{ ...").is_err());
    }

    #[test]
    fn test_static() {
        assert!(check("... static ...").is_err());
        assert!(check("static ...").is_err());
    }

    #[test]
    fn test_static_lifetime() {
        assert!(check("... 'static ...").is_ok());
    }

    #[test]
    fn test_extern() {
        assert!(check("... extern ...").is_err());
    }

    #[test]
    fn test_crate() {
        assert!(check("... crate ...").is_err());
    }

    #[test]
    fn test_macros() {
        assert!(check("... macro_rules! ...").is_err());
        assert!(check("... include! ...").is_err());
        assert!(check("... include_bytes! ...").is_err());
        assert!(check("... include_str! ...").is_err());
    }

    #[test]
    fn test_inside_words() {
        assert!(check("... foounsafe {} ...").is_ok());
        assert!(check("... unsafefoo {} ...").is_ok());
        assert!(check("... staticfoo {} ...").is_ok());
        assert!(check("... externfoo {} ...").is_ok());
        assert!(check("... cratefoo {} ...").is_ok());
    }
}
