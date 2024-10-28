use anyhow::bail;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Multifile {
    pub src: String,
    pub filenames: Vec<String>,
}

impl Multifile {
    pub fn empty() -> Self {
        Self {
            src: String::new(),
            filenames: Vec::new(),
        }
    }

    pub fn finalize(&self, main_filename: &str) -> anyhow::Result<String> {
        if !self.filenames.contains(&main_filename.to_owned()) {
            bail!("Main file {} not found in multifile", main_filename);
        }
        let mut src = self.src.clone();
        if main_filename != "lib.rs" && self.filenames.len() > 1 {
            src.push_str(&format!(
                "\npub use {}::*;\n",
                main_filename.strip_suffix(".rs").unwrap()
            ));
        }
        Ok(src)
    }
}

// 3 cases:
// 1. Single file with arbitrary name.
// 2. Multiple files with Ship in lib.rs.
// 3. Multiple files with Ship in a child module.
pub fn join(mut files: HashMap<String, String>) -> Result<Multifile, anyhow::Error> {
    if files.len() == 1 {
        let (filename, src) = files.drain().next().unwrap();
        return Ok(Multifile {
            src,
            filenames: vec![filename],
        });
    }

    let lib = if let Some(src) = files.get("lib.rs") {
        src.clone()
    } else {
        bail!("Missing lib.rs file");
    };

    let re = regex::Regex::new(r"(pub )?mod (\w+);").unwrap();
    let src = re
        .replace_all(&lib, |caps: &regex::Captures| {
            let pubk = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let name = caps.get(2).unwrap().as_str();
            let filename = format!("{}.rs", name);
            if let Some(src) = files.get(&filename) {
                format!(
                    "{}mod {} {{ // start multifile\n{}\n}} // end multifile",
                    pubk, name, src
                )
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        })
        .into_owned();

    let mut filenames: Vec<String> = files.keys().cloned().collect();
    filenames.sort();
    Ok(Multifile { src, filenames })
}

pub fn split(lib: &str) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let re = regex::Regex::new(
        r"(pub )?mod (\w+) \{ // start multifile\n(?s:(.*?))\n\} // end multifile",
    )
    .unwrap();
    for caps in re.captures_iter(lib) {
        let name = caps.get(2).unwrap().as_str();
        let src = caps.get(3).unwrap().as_str();
        files.insert(format!("{}.rs", name), src.to_string());
    }
    let lib = re.replace_all(lib, "${1}mod $2;").into_owned();
    files.insert("lib.rs".to_string(), lib);
    files
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn test_join_single_files() {
        let mut files = std::collections::HashMap::new();
        files.insert("foo.rs".to_string(), "fn foo() {}".to_string());
        let multifile = super::join(files).unwrap();
        assert_eq!(multifile.filenames, vec!["foo.rs"]);
        assert_eq!(multifile.finalize("foo.rs").unwrap(), "fn foo() {}");
    }

    #[test]
    fn test_join_multiple_files() {
        let mut files = std::collections::HashMap::new();
        files.insert("lib.rs".to_string(), "mod foo;\npub mod bar;\n".to_string());
        files.insert("foo.rs".to_string(), "fn foo() {}".to_string());
        files.insert("bar.rs".to_string(), "fn bar() {}".to_string());
        let multifile = super::join(files).unwrap();
        assert_eq!(multifile.filenames, vec!["bar.rs", "foo.rs", "lib.rs"]);

        assert_eq!(
            multifile.finalize("foo.rs").unwrap(),
            "\
mod foo { // start multifile
fn foo() {}
} // end multifile
pub mod bar { // start multifile
fn bar() {}
} // end multifile

pub use foo::*;
"
        );

        assert_eq!(
            multifile.finalize("lib.rs").unwrap(),
            "\
mod foo { // start multifile
fn foo() {}
} // end multifile
pub mod bar { // start multifile
fn bar() {}
} // end multifile
"
        );
    }

    #[test]
    fn test_join_no_lib() {
        let mut files = std::collections::HashMap::new();
        files.insert(
            "mainfile.rs".to_string(),
            "mod foo;\npub mod bar;\npub struct Ship {}\n".to_string(),
        );
        files.insert("foo.rs".to_string(), "fn foo() {}".to_string());
        files.insert("bar.rs".to_string(), "fn bar() {}".to_string());
        assert_eq!(
            super::join(files).unwrap_err().to_string(),
            "Missing lib.rs file"
        );
    }

    #[test]
    fn test_join_missing_file() {
        let mut files = std::collections::HashMap::new();
        files.insert("lib.rs".to_string(), "mod foo;".to_string());
        files.insert("bar.rs".to_string(), "".to_string());
        assert_eq!(
            super::join(files).unwrap().finalize("lib.rs").unwrap(),
            "mod foo;"
        );
    }

    #[test]
    fn test_split() {
        let lib = "pub mod foo { // start multifile\nfn foo() {}\n} // end multifile";
        let mut files = std::collections::HashMap::new();
        files.insert("lib.rs".to_string(), "pub mod foo;".to_string());
        files.insert("foo.rs".to_string(), "fn foo() {}".to_string());
        assert_eq!(canonicalize(&super::split(lib)), canonicalize(&files));
    }

    #[test]
    fn test_roundtrip() {
        let mut files = std::collections::HashMap::new();
        let reference = include_str!("../../builtin_ai/src/reference.rs");
        //let reference = "fn baz() {}\nfn boo() {}\n";
        let lib = "mod foo;\nmod bar;\n".to_string();
        files.insert("lib.rs".to_string(), lib.clone());
        files.insert("foo.rs".to_string(), reference.to_string());
        files.insert("bar.rs".to_string(), reference.to_string());

        let multifile = super::join(files.clone())
            .unwrap()
            .finalize("lib.rs")
            .unwrap();
        eprintln!("---------");
        eprintln!("{}", multifile);
        eprintln!("---------");
        let mut splitfiles = super::split(&multifile);
        splitfiles.insert("lib.rs".to_string(), lib);
        assert_eq!(canonicalize(&splitfiles), canonicalize(&files));
    }

    fn canonicalize(map: &HashMap<String, String>) -> Vec<(String, String)> {
        let mut v: Vec<_> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        v.sort();
        v
    }
}
