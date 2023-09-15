use serde::{Deserialize, Serialize};

pub fn read<T: for<'a> Deserialize<'a>>(name: &str, default: T) -> T {
    fn read_setting_inner<T: for<'a> Deserialize<'a>>(name: &str) -> Option<T> {
        let storage = web_sys::window()?.local_storage().ok()??;
        let value = storage.get_item(name).ok()??;
        serde_json::from_str(&value).ok()
    }
    read_setting_inner(name).unwrap_or(default)
}

pub fn write<T: Serialize>(name: &str, value: &T) {
    fn write_setting_inner<T: Serialize>(name: &str, value: &T) -> Option<()> {
        let storage = web_sys::window()?.local_storage().ok()??;
        let value = serde_json::to_string(value).ok()?;
        storage.set_item(name, &value).ok()?;
        Some(())
    }
    if write_setting_inner(name, value).is_none() {
        log::warn!("Failed to write setting {}", name);
    }
}
