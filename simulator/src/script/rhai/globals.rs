use rhai::plugin::*;
use smartstring::alias::CompactString;

#[export_module]
pub mod plugin {
    #[derive(Copy, Clone)]
    pub struct Globals {
        pub map: *mut std::collections::HashMap<CompactString, Dynamic>,
    }

    #[rhai_fn(index_get, return_raw)]
    pub fn get(obj: Globals, key: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        unsafe {
            match (*obj.map).get(key).cloned() {
                Some(value) => Ok(value),
                None => Err(format!("unknown global variable {:?}", key).into()),
            }
        }
    }

    #[rhai_fn(index_set)]
    pub fn set(obj: Globals, key: &str, value: Dynamic) {
        unsafe {
            (*obj.map).insert(key.into(), value);
        }
    }
}
