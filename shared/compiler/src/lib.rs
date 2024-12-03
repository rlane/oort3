mod sanitizer;

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub struct Compiler {
    #[allow(dead_code)]
    tmp_dir: Option<tempdir::TempDir>,
    dir: PathBuf,
    offline: bool,
    rustc: String,
}

#[allow(clippy::new_without_default)]
impl Compiler {
    pub fn new() -> Compiler {
        let tmp_dir = tempdir::TempDir::new("oort_compiler").unwrap();
        let dir = tmp_dir.path().to_path_buf();
        Self {
            tmp_dir: Some(tmp_dir),
            dir,
            offline: true,
            rustc: find_rustc(),
        }
    }

    pub fn new_with_dir(dir: &Path) -> Compiler {
        Self {
            tmp_dir: None,
            dir: dir.to_path_buf(),
            offline: true,
            rustc: find_rustc(),
        }
    }

    pub fn enable_online(&mut self) {
        self.offline = false;
    }

    pub fn compile(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        match detect_language(code) {
            Language::Rust => self.compile_rust(code),
            Language::Cpp => self.compile_cpp(code),
            Language::Unknown => bail!("Unknown language"),
        }
    }

    pub fn compile_rust(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        let tmp_path = &self.dir;

        // TODO return BAD_REQUEST on failure
        sanitizer::check(code)?;

        if std::fs::metadata(tmp_path.join("Cargo.toml")).is_ok() {
            return self.compile_rust_fast(code);
        }

        std::fs::write(
            tmp_path.join("rust-toolchain.toml"),
            include_bytes!("../../../rust-toolchain.toml"),
        )?;
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
        std::fs::write(
            tmp_path.join("api/src/panic.rs"),
            include_bytes!("../../api/src/panic.rs"),
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
        std::fs::write(
            tmp_path.join("ai/src/tick.rs"),
            include_bytes!("../../ai/src/tick.rs"),
        )?;
        std::fs::write(tmp_path.join("ai/src/user.rs"), code.as_bytes())?;

        let disallowed_environment_variables = ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"];

        match std::process::Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                tmp_path.join("Cargo.toml").as_os_str().to_str().unwrap(),
                "--target-dir",
                tmp_path.join("target").as_os_str().to_str().unwrap(),
                "-v",
                "-j1",
                if self.offline { "--offline" } else { "-v" },
                "--release",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .current_dir(tmp_path)
            .env_clear()
            .envs(
                std::env::vars()
                    .filter(|(k, _)| !disallowed_environment_variables.contains(&k.as_str())),
            )
            .env(
                "RUSTFLAGS",
                "-C opt-level=s -C link-arg=-zstack-size=16384 -C llvm-args=-rng-seed=42",
            )
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    log::error!("stdout:\n{}", std::str::from_utf8(&output.stdout)?);
                    log::error!("stderr:\n{}", std::str::from_utf8(&output.stderr)?);
                    bail!("cargo failed: {}", std::str::from_utf8(&output.stderr)?);
                }
            }
            Err(e) => {
                log::error!("spawning cargo failed: {}", e);
                bail!("spawning cargo failed: {}", e);
            }
        }

        self.compile_rust_fast(code)
    }

    pub fn compile_rust_fast(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        let tmp_path = &self.dir;
        std::fs::write(tmp_path.join("ai/src/user.rs"), code.as_bytes())?;
        let rustc_bin_dir = Path::new(&self.rustc).parent().unwrap();
        let mut ld_library_path = format!("{}/../lib", rustc_bin_dir.display(),);
        if std::env::var("LD_LIBRARY_PATH").is_ok() {
            ld_library_path = format!(
                "{}:{}",
                ld_library_path,
                std::env::var("LD_LIBRARY_PATH").unwrap()
            );
        }

        let output = std::process::Command::new(&self.rustc)
            .current_dir(tmp_path)
            .env("LD_LIBRARY_PATH", &ld_library_path)
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

    pub fn compile_cpp(&mut self, code: &str) -> Result<Vec<u8> /* wasm */> {
        let tmp_path = &self.dir;
        let src_path = tmp_path.join("user.cpp");
        let dst_path = tmp_path.join("user.wasm");
        std::fs::write(&src_path, code.as_bytes())?;
        std::fs::write(
            tmp_path.join("oort.h"),
            include_bytes!("../../cpp-api/oort.h"),
        )?;
        std::fs::write(
            tmp_path.join("oort.cpp"),
            include_bytes!("../../cpp-api/oort.cpp"),
        )?;

        let output = std::process::Command::new("zig")
            .args([
                "c++",
                "-shared",
                "-target",
                "wasm32-wasi",
                "-fno-exceptions",
                "-fno-stack-protector",
                "-Oz",
                "-Wl,--export=SYSTEM_STATE",
                "-Wl,--export=ENVIRONMENT",
                "-Wl,--export=PANIC_BUFFER",
                tmp_path.join("oort.cpp").as_os_str().to_str().unwrap(),
                src_path.as_os_str().to_str().unwrap(),
                "-o",
                dst_path.as_os_str().to_str().unwrap(),
            ])
            .output()?;
        if !output.status.success() {
            bail!(
                "compilation failed: {}",
                std::str::from_utf8(&output.stderr)?
            );
        }

        let output = std::process::Command::new("wasm-strip")
            .args([dst_path.as_os_str().to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            bail!(
                "wasm-strip failed: {}",
                std::str::from_utf8(&output.stderr)?
            );
        }

        Ok(std::fs::read(&dst_path)?)
    }
}

fn find_rlib(tmp_path: &Path, crate_name: &str) -> PathBuf {
    if let Some(path) = glob::glob(
        tmp_path
            .join(format!(
                "target/wasm32-unknown-unknown/release/deps/lib{crate_name}-*.rlib"
            ))
            .as_os_str()
            .to_str()
            .unwrap(),
    )
    .unwrap()
    .next()
    {
        return path.unwrap();
    }
    panic!("{crate_name} rlib not found");
}

fn find_rustc() -> String {
    let output = std::process::Command::new("rustup")
        .args(["which", "rustc"])
        .output()
        .unwrap();
    if output.status.success() {
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .trim()
            .to_string()
    } else {
        log::error!(
            "Failed to find rustc: {}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
        "rustc".to_string()
    }
}

enum Language {
    Rust,
    Cpp,
    Unknown,
}

fn detect_language(code: &str) -> Language {
    log::debug!("Code: {:?}", code);
    if code.contains("#include") {
        log::info!("Detected C++");
        Language::Cpp
    } else if code.contains("impl Ship") {
        Language::Rust
    } else {
        Language::Unknown
    }
}
