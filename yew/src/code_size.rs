use flate2::write::DeflateEncoder;
use flate2::Compression;
use no_comment::{languages, IntoWithoutComments as _};
use std::io::Write;

pub fn calculate(code: &str) -> usize {
    let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
    e.write_all(
        code.chars()
            .without_comments(languages::rust())
            .collect::<String>()
            .as_bytes(),
    )
    .expect("compression failed");
    e.finish().expect("compression failed").len()
}

#[cfg(test)]
mod test {
    use super::calculate;

    #[test]
    fn test_comment() {
        assert_eq!(calculate("// test abcd"), 2);
        assert_eq!(calculate("/* test abcd */"), 2);
    }
}
