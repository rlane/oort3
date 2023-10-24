use crate::{error, Error};
use http::StatusCode;
use lazy_static::lazy_static;
use regex::Regex;

pub fn check(text: &str) -> Result<(), Error> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"\b(macro_rules|include|include_bytes|include_str)(\b|!)"#).unwrap();
    }
    if let Some(m) = RE.find(text) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            format!("Code did not pass sanitizer (found {:?})", m.as_str()),
        ));
    }

    lazy_static! {
        static ref RE2: Regex = Regex::new(r#"#\[[^]]*path"#).unwrap();
    }
    if let Some(m) = RE2.find(text) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            format!("Code did not pass sanitizer (found {:?})", m.as_str()),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_macros() {
        assert!(check("... macro_rules! ...").is_err());
        assert!(check("... include! ...").is_err());
        assert!(check("... include_bytes! ...").is_err());
        assert!(check("... include_str! ...").is_err());
    }

    #[test]
    fn path_attr() {
        assert!(check("... #[path = \"/dev/random\"] ...").is_err());
        assert!(check("... #[\npath = \"/dev/random\"] ...").is_err());
        assert!(check("... #[\t  path\n= \"/dev/random\"] ...").is_err());
    }
}
