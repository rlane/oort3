use crate::simulation::Code;

struct BuiltinScript {
    name: &'static str,
    source_code: &'static str,
    compiled_code: &'static [u8],
}

macro_rules! builtin {
    ($name:expr) => {
        BuiltinScript {
            name: $name,
            source_code: include_str!(concat!("../../../ai/", $name, ".rs")),
            compiled_code: include_bytes!(concat!("../../../ai/compiled/", $name, ".wasm")),
        }
    };
}

const BUILTIN_SCRIPTS: &[BuiltinScript] = &[
    builtin!("gunnery"),
    builtin!("tutorial/tutorial09.enemy"),
    builtin!("tutorial/tutorial08.solution"),
    builtin!("tutorial/tutorial09.solution"),
    builtin!("tutorial/tutorial05.enemy"),
    builtin!("tutorial/tutorial01.initial"),
    builtin!("tutorial/tutorial07.enemy"),
    builtin!("tutorial/tutorial06.initial"),
    builtin!("tutorial/tutorial03.solution"),
    builtin!("tutorial/tutorial11.solution"),
    builtin!("tutorial/tutorial11.enemy"),
    builtin!("tutorial/tutorial03.initial"),
    builtin!("tutorial/tutorial04.solution"),
    builtin!("tutorial/tutorial02.initial"),
    builtin!("tutorial/tutorial11.initial"),
    builtin!("tutorial/tutorial08.enemy"),
    builtin!("tutorial/tutorial02.solution"),
    builtin!("tutorial/tutorial07.solution"),
    builtin!("tutorial/tutorial09.initial"),
    builtin!("tutorial/tutorial07.initial"),
    builtin!("tutorial/tutorial08.initial"),
    builtin!("tutorial/tutorial05.solution"),
    builtin!("tutorial/tutorial01.solution"),
    builtin!("tutorial/tutorial10.solution"),
    builtin!("tutorial/tutorial06.enemy"),
    builtin!("tutorial/tutorial06.solution"),
    builtin!("tutorial/tutorial10.initial"),
    builtin!("tutorial/tutorial04.initial"),
    builtin!("tutorial/tutorial05.initial"),
    builtin!("tutorial/tutorial10.enemy"),
    builtin!("reference"),
    builtin!("missile"),
];

fn lookup(name: &str) -> Option<&'static BuiltinScript> {
    BUILTIN_SCRIPTS.iter().find(|x| x.name == name)
}

fn rust(code: &str) -> Code {
    Code::Rust(code.to_string())
}

fn wasm(code: &[u8]) -> Code {
    Code::Wasm(code.to_vec())
}

pub fn load_source(name: &str) -> Result<Code, String> {
    match lookup(name) {
        Some(builtin_script) => Ok(rust(builtin_script.source_code)),
        _ => Err(format!("Unknown builtin script {name:?}")),
    }
}

pub fn load_compiled(name: &str) -> Result<Code, String> {
    match lookup(name) {
        Some(builtin_script) => Ok(wasm(builtin_script.compiled_code)),
        _ => Err(format!("Unknown builtin script {name:?}")),
    }
}
