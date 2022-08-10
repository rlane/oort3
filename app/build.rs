const GIT_DIR: &str = "../../.git";

fn main() {
    println!("cargo:rerun-if-changed=../Cargo.toml");
    println!("cargo:rerun-if-changed={GIT_DIR}/HEAD");
    println!("cargo:rerun-if-changed={GIT_DIR}/refs");
    println!("cargo:rerun-if-changed={GIT_DIR}/index");
    built::write_built_file().expect("Failed to acquire build-time information");
}
