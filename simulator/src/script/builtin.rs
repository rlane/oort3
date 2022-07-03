use crate::simulation::Code;
use libflate::gzip::Decoder;
use std::io::Read;
use tar::Archive;

fn builtin_ai_data() -> &'static [u8] {
    include_bytes!("../../../ai/builtin-ai.tar.gz")
}

pub fn load_source(name: &str) -> Result<Code, String> {
    let name = format!("{}.rs", name);
    let mut a = Archive::new(Decoder::new(builtin_ai_data()).unwrap());
    for file in a.entries().unwrap() {
        let mut file = file.unwrap();
        let filename = file.header().path().unwrap();
        if filename.as_ref().to_str().unwrap() == name {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            return Ok(Code::Rust(s));
        }
    }

    Err(format!("Unknown builtin script {name:?}"))
}

pub fn load_compiled(name: &str) -> Result<Code, String> {
    let name = format!("{}.wasm", name);
    let mut a = Archive::new(Decoder::new(builtin_ai_data()).unwrap());
    for file in a.entries().unwrap() {
        let mut file = file.unwrap();
        let filename = file.header().path().unwrap();
        if filename.as_ref().to_str().unwrap() == name {
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data).unwrap();
            return Ok(Code::Wasm(data));
        }
    }

    Err(format!("Unknown builtin script {name:?}"))
}
