pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn version() -> String {
    let mut fragments = vec![built_info::GIT_VERSION.unwrap_or("unknown")];
    if built_info::GIT_DIRTY == Some(true) {
        fragments.push("dirty");
    }
    fragments.join("-")
}
