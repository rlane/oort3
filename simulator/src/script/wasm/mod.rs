use super::{ShipController, TeamController};
use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use js_sys::{Float64Array, Function, Object, Reflect, Uint8Array, WebAssembly};
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
        handle: ShipHandle,
        sim: &mut Simulation,
        _orders: String,
    ) -> Result<Box<dyn ShipController>, super::Error> {
        let instance: WebAssembly::Instance =
            WebAssembly::Instance::new(&self.module, &Object::new())?;
        let exports = instance.exports();

        let system_state_global: WebAssembly::Global =
            Reflect::get(exports.as_ref(), &"SYSTEM_STATE".into())?.dyn_into()?;
        let system_state_offset = system_state_global.value().as_f64().unwrap() as u32;

        let memory: WebAssembly::Memory =
            Reflect::get(exports.as_ref(), &"memory".into())?.dyn_into()?;
        let buffer = memory.buffer();
        let system_state =
            Float64Array::new_with_byte_offset_and_length(&buffer, system_state_offset, 2);

        let tick = Reflect::get(exports.as_ref(), &"export_tick".into())?
            .dyn_into::<Function>()
            .expect("export_tick wasn't a function");

        Ok(Box::new(WasmShipController {
            handle,
            sim,
            system_state,
            tick,
        }))
    }
}

struct WasmShipController {
    pub handle: ShipHandle,
    pub sim: *mut Simulation,
    pub system_state: Float64Array,
    pub tick: Function,
}

impl ShipController for WasmShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        //log::info!("before: system state: {:?}", self.system_state.to_vec());
        self.tick.call0(&JsValue::undefined())?;
        //log::info!("after:  system state: {:?}", self.system_state.to_vec());
        let system_state = self.system_state.to_vec();
        let sim = unsafe { &mut *self.sim };
        sim.ship_mut(self.handle)
            .accelerate(Vec2::new(system_state[0], system_state[1]));
        Ok(())
    }

    fn write_target(&mut self, _target: Vec2) {}
}
