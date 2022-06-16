use super::{ShipController, TeamController};
use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use js_sys::{Function, Object, Reflect, Uint8Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const WASM: &[u8] = include_bytes!("../../../../ai/rust/reference.wasm");

pub type Vec2 = nalgebra::Vector2<f64>;

pub struct WasmTeamController {
    pub module: WebAssembly::Module,
}

impl WasmTeamController {
    pub fn create() -> Result<Box<dyn TeamController>, super::Error> {
        log::info!("Creating WasmTeamController");
        let buffer = Uint8Array::new_with_length(WASM.len() as u32);
        buffer.copy_from(WASM);
        let module: WebAssembly::Module = WebAssembly::Module::new(&buffer)?;

        Ok(Box::new(WasmTeamController { module }))
    }
}

impl TeamController for WasmTeamController {
    fn create_ship_controller(
        &mut self,
        _handle: ShipHandle,
        _sim: &mut Simulation,
        _orders: String,
    ) -> Result<Box<dyn ShipController>, super::Error> {
        let instance: WebAssembly::Instance =
            WebAssembly::Instance::new(&self.module, &Object::new())?;
        Ok(Box::new(WasmShipController { instance }))
    }
}

struct WasmShipController {
    pub instance: WebAssembly::Instance,
}

impl ShipController for WasmShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        let exports = self.instance.exports();
        let tick = Reflect::get(exports.as_ref(), &"export_tick".into())?
            .dyn_into::<Function>()
            .expect("export_tick wasn't a function");
        tick.call0(&JsValue::undefined())?;
        Ok(())
    }

    fn write_target(&mut self, _target: Vec2) {}
}
