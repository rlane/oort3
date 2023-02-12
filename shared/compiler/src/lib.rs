use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub struct Compiler {
    #[allow(dead_code)]
    tmp_dir: Option<tempdir::TempDir>,
    dir: PathBuf,
}

impl Compiler {
    pub fn new() -> Compiler {
        let tmp_dir = tempdir::TempDir::new("oort_compiler").unwrap();
        let dir = tmp_dir.path().to_path_buf();
        Self {
            tmp_dir: Some(tmp_dir),
            dir,
        }
    }

    pub fn new_with_dir(dir: &Path) -> Compiler {
        Self {
            tmp_dir: None,
            dir: dir.to_path_buf(),
        }
    }

    pub fn compile(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        let tmp_path = &self.dir;

        if std::fs::metadata(tmp_path.join("Cargo.toml")).is_ok() {
            return self.compile_fast(code);
        }

        std::fs::write(
            tmp_path.join("Cargo.toml"),
            include_bytes!("../../../Cargo.toml.user"),
        )?;
        std::fs::write(
            tmp_path.join("Cargo.lock"),
            include_bytes!("../../../Cargo.lock.user"),
        )?;
        std::fs::create_dir_all(tmp_path.join("api/src"))?;
        std::fs::write(
            tmp_path.join("api/Cargo.toml"),
            include_bytes!("../../api/Cargo.toml"),
        )?;
        std::fs::write(tmp_path.join("api/README.md"), b"")?;
        std::fs::write(
            tmp_path.join("api/src/lib.rs"),
            include_bytes!("../../api/src/lib.rs"),
        )?;
        std::fs::write(
            tmp_path.join("api/src/vec.rs"),
            include_bytes!("../../api/src/vec.rs"),
        )?;

        std::fs::create_dir_all(tmp_path.join("ai/src"))?;
        std::fs::write(
            tmp_path.join("ai/Cargo.toml"),
            include_bytes!("../../ai/Cargo.toml"),
        )?;
        std::fs::write(
            tmp_path.join("ai/src/lib.rs"),
            include_bytes!("../../ai/src/lib.rs"),
        )?;
        std::fs::write(tmp_path.join("ai/src/user.rs"), code.as_bytes())?;

        let output = std::process::Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                tmp_path.join("Cargo.toml").as_os_str().to_str().unwrap(),
                "--target-dir",
                tmp_path.join("target").as_os_str().to_str().unwrap(),
                "-v",
                "-j1",
                "--offline",
                "--release",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .env(
                "RUSTFLAGS",
                "-C opt-level=s -C link-arg=-zstack-size=16384 -C llvm-args=-rng-seed=42",
            )
            .output()?;
        if !output.status.success() {
            bail!("cargo failed: {}", std::str::from_utf8(&output.stderr)?);
        }

        self.compile_fast(code)
    }

    pub fn compile_fast(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        let tmp_path = &self.dir;
        std::fs::write(tmp_path.join("ai/src/user.rs"), code.as_bytes())?;

        let output = std::process::Command::new("rustc")
            .args([
                "--crate-name",
                "oort_ai",
                "--edition=2021",
                tmp_path.join("ai/src/lib.rs").as_os_str().to_str().unwrap(),
                "--crate-type",
                "cdylib",
                "-o",
                tmp_path
                    .join("target/wasm32-unknown-unknown/release/oort_ai.wasm")
                    .as_os_str()
                    .to_str()
                    .unwrap(),
                "--target",
                "wasm32-unknown-unknown",
                "-C",
                "strip=debuginfo",
                "-L",
                &format!(
                    "dependency={}",
                    tmp_path
                        .join("target/wasm32-unknown-unknown/release/deps")
                        .as_os_str()
                        .to_str()
                        .unwrap()
                ),
                "--extern",
                &format!(
                    "oort_api={}",
                    find_rlib(tmp_path, "oort_api")
                        .as_os_str()
                        .to_str()
                        .unwrap()
                ),
                "-C",
                "opt-level=s",
                "-C",
                "link-arg=-zstack-size=16384",
                "-C",
                "llvm-args=-rng-seed=42",
                "--remap-path-prefix",
                &format!("{}=/tmp/oort-ai", tmp_path.display()),
            ])
            .output()?;
        if !output.status.success() {
            bail!("rustc failed: {}", std::str::from_utf8(&output.stderr)?);
        }

        Ok(std::fs::read(tmp_path.join(
            "target/wasm32-unknown-unknown/release/oort_ai.wasm",
        ))?)
    }
}

fn find_rlib(tmp_path: &Path, crate_name: &str) -> PathBuf {
    for entry in glob::glob(
        tmp_path
            .join(format!(
                "target/wasm32-unknown-unknown/release/deps/lib{crate_name}-*.rlib"
            ))
            .as_os_str()
            .to_str()
            .unwrap(),
    )
    .unwrap()
    {
        return entry.unwrap();
    }
    panic!("{crate_name} rlib not found");
}
