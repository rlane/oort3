pub mod shared;

use super::{ShipController, TeamController};
use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use shared::*;
use wasmer::{imports, Instance, Module, Store, WasmPtr};

const WASM: &[u8] = include_bytes!("../../../../ai/rust/reference.wasm");

pub type Vec2 = nalgebra::Vector2<f64>;

pub struct WasmTeamController {
    pub module: Module,
}

impl WasmTeamController {
    pub fn create() -> Result<Box<dyn TeamController>, super::Error> {
        log::info!("Creating WasmTeamController");
        let store = Store::default();
        let module = translate_error(Module::new(&store, WASM))?;

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
        let import_object = imports! {};
        let instance = Instance::new(&self.module, &import_object)?;

        let memory = translate_error(instance.exports.get_memory("memory"))?.clone();
        let system_state_offset: i32 =
            translate_error(instance.exports.get_global("SYSTEM_STATE"))?
                .get()
                .i32()
                .unwrap();
        let system_state_ptr: WasmPtr<f64> = WasmPtr::new(system_state_offset as u32);

        let tick = translate_error(instance.exports.get_function("export_tick"))?.clone();

        Ok(Box::new(WasmShipController {
            handle,
            sim,
            memory,
            system_state_ptr,
            tick,
        }))
    }
}

struct WasmShipController {
    pub handle: ShipHandle,
    pub sim: *mut Simulation,
    pub memory: wasmer::Memory,
    pub system_state_ptr: WasmPtr<f64>,
    pub tick: wasmer::Function,
}

impl WasmShipController {
    pub fn read_system_state(&self) -> [f64; SystemState::Size as usize] {
        let mut system_state = [0.0; SystemState::Size as usize];
        let mut ptr = self.system_state_ptr;
        for i in 0..SystemState::Size as usize {
            system_state[i] = ptr.deref(&self.memory).unwrap().get();
            ptr = WasmPtr::new(ptr.offset() + 8);
        }
        system_state
    }
}

impl ShipController for WasmShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        //log::info!("before: system state: {:?}", self.read_system_state());
        translate_error(self.tick.call(&[]))?;
        //log::info!("after:  system state: {:?}", self.read_system_state());
        let system_state = self.read_system_state();
        let sim = unsafe { &mut *self.sim };

        sim.ship_mut(self.handle).accelerate(Vec2::new(
            system_state[SystemState::AccelerateX as usize],
            system_state[SystemState::AccelerateY as usize],
        ));

        sim.ship_mut(self.handle)
            .torque(system_state[SystemState::Torque as usize]);

        Ok(())
    }

    fn write_target(&mut self, _target: Vec2) {}
}

fn translate_error<T, U>(err: Result<T, U>) -> Result<T, super::Error>
where
    U: std::fmt::Debug,
{
    match err {
        Ok(val) => Ok(val),
        Err(err) => Err(super::Error {
            line: 0,
            msg: format!("Wasmer error: {:?}", err),
        }),
    }
}
