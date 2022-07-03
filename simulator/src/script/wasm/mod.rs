// TODO:
// - Debug log
// - Debug lines
// - Vec2 methods and overloads
// - Angle utilities
// - RNG

use super::{ShipController, TeamController};
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::Simulation;
use oort_shared::*;
use wasmer::{imports, Instance, Module, Store, WasmPtr};

pub type Vec2 = nalgebra::Vector2<f64>;

pub struct WasmTeamController {
    pub module: Module,
}

impl WasmTeamController {
    pub fn create(code: &[u8]) -> Result<Box<dyn TeamController>, super::Error> {
        log::info!("Creating WasmTeamController");
        #[cfg(not(target_arch = "wasm32"))]
        let store = Store::new_with_engine(
            &wasmer_compiler::Universal::new(wasmer_compiler_cranelift::Cranelift::default())
                .engine(),
        );
        #[cfg(target_arch = "wasm32")]
        let store = Store::default();
        let module = translate_error(Module::new(&store, code))?;

        Ok(Box::new(WasmTeamController { module }))
    }
}

impl TeamController for WasmTeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: &mut Simulation,
        orders: String,
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

        let ctrl = WasmShipController {
            handle,
            sim,
            memory,
            system_state_ptr,
            tick,
        };

        let mut state = ctrl.read_system_state();
        state.set(
            SystemState::Seed,
            (make_seed(sim.seed(), handle) & 0xffffff) as f64,
        );
        if let Ok(orders) = orders.parse::<f64>() {
            state.set(SystemState::Orders, orders);
        }
        if let Some(radar) = sim.ship(handle).data().radar.as_ref() {
            state.set(SystemState::RadarHeading, radar.heading);
            state.set(SystemState::RadarWidth, radar.width);
        }
        ctrl.write_system_state(&state);

        Ok(Box::new(ctrl))
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
    pub fn read_system_state(&self) -> LocalSystemState {
        let mut state = [0.0; SystemState::Size as usize];
        let slice = self
            .system_state_ptr
            .slice(&self.memory, SystemState::Size as u32)
            .expect("system state read");
        slice.read_slice(&mut state).expect("system state read");
        LocalSystemState { state }
    }

    pub fn write_system_state(&self, state: &LocalSystemState) {
        let slice = self
            .system_state_ptr
            .slice(&self.memory, SystemState::Size as u32)
            .expect("system state write");
        slice.write_slice(&state.state).expect("system state write");
    }

    pub fn read_string(&self, offset: u32, length: u32) -> Option<String> {
        let ptr: WasmPtr<u8> = WasmPtr::new(offset);
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(length as usize, 0);
        let slice = ptr.slice(&self.memory, length).ok()?;
        slice.read_slice(&mut bytes).ok()?;
        String::from_utf8(bytes).ok()
    }
}

fn make_seed(sim_seed: u32, handle: ShipHandle) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut s = DefaultHasher::new();
    let (i, j) = handle.0.into_raw_parts();
    s.write_u32(sim_seed);
    s.write_u32(i);
    s.write_u32(j);
    s.finish() as i64
}

impl ShipController for WasmShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        {
            let mut state = self.read_system_state();
            let sim = unsafe { &mut *self.sim };

            state.set(
                SystemState::Class,
                translate_class(sim.ship(self.handle).data().class) as u32 as f64,
            );

            let position = sim.ship(self.handle).position();
            state.set(SystemState::PositionX, position.x);
            state.set(SystemState::PositionY, position.y);

            let velocity = sim.ship(self.handle).velocity();
            state.set(SystemState::VelocityX, velocity.x);
            state.set(SystemState::VelocityY, velocity.y);

            state.set(SystemState::Heading, sim.ship(self.handle).heading());
            state.set(
                SystemState::AngularVelocity,
                sim.ship(self.handle).angular_velocity(),
            );

            if let Some(radar) = sim.ship(self.handle).radar() {
                state.set(SystemState::RadarHeading, radar.heading);
                state.set(SystemState::RadarWidth, radar.width);
            }

            if let Some(contact) = crate::radar::scan(sim, self.handle) {
                state.set(SystemState::RadarContactFound, 1.0);
                state.set(SystemState::RadarContactPositionX, contact.position.x);
                state.set(SystemState::RadarContactPositionY, contact.position.y);
                state.set(SystemState::RadarContactVelocityX, contact.velocity.x);
                state.set(SystemState::RadarContactVelocityY, contact.velocity.y);
                if let Some(class) = contact.class {
                    state.set(
                        SystemState::RadarContactClass,
                        translate_class(class) as u32 as f64,
                    );
                } else {
                    state.set(SystemState::RadarContactClass, Class::Unknown as u32 as f64);
                }
            } else {
                state.set(SystemState::RadarContactFound, 0.0);
            }

            self.write_system_state(&state);
        }

        translate_error(self.tick.call(&[]))?;

        {
            let mut state = self.read_system_state();
            let sim = unsafe { &mut *self.sim };

            sim.ship_mut(self.handle).accelerate(Vec2::new(
                state.get(SystemState::AccelerateX),
                state.get(SystemState::AccelerateY),
            ));
            state.set(SystemState::AccelerateX, 0.0);
            state.set(SystemState::AccelerateY, 0.0);

            sim.ship_mut(self.handle)
                .torque(state.get(SystemState::Torque));
            state.set(SystemState::Torque, 0.0);

            for (i, (aim, fire)) in [
                (SystemState::Gun0Aim, SystemState::Gun0Fire),
                (SystemState::Gun1Aim, SystemState::Gun1Fire),
                (SystemState::Gun2Aim, SystemState::Gun2Fire),
                (SystemState::Gun3Aim, SystemState::Gun3Fire),
            ]
            .iter()
            .enumerate()
            {
                if state.get(*fire) > 0.0 {
                    sim.ship_mut(self.handle).aim_gun(i as i64, state.get(*aim));
                    sim.ship_mut(self.handle).fire_gun(i as i64);
                    state.set(*fire, 0.0);
                }
            }

            for (i, (launch, orders)) in [
                (SystemState::Missile0Launch, SystemState::Missile0Orders),
                (SystemState::Missile1Launch, SystemState::Missile1Orders),
                (SystemState::Missile2Launch, SystemState::Missile2Orders),
                (SystemState::Missile3Launch, SystemState::Missile3Orders),
            ]
            .iter()
            .enumerate()
            {
                if state.get(*launch) > 0.0 {
                    let orders = state.get(*orders);
                    sim.ship_mut(self.handle)
                        .launch_missile(i as i64, orders.to_string());
                    state.set(*launch, 0.0);
                }
            }

            if let Some(radar) = sim.ship_mut(self.handle).data_mut().radar.as_mut() {
                radar.heading = state.get(SystemState::RadarHeading);
                radar.width = state.get(SystemState::RadarWidth);
            }

            if state.get(SystemState::Explode) > 0.0 {
                sim.ship_mut(self.handle).explode();
                state.set(SystemState::Explode, 0.0);
            }

            if state.get(SystemState::DebugTextLength) > 0.0 {
                let offset = state.get(SystemState::DebugTextPointer) as u32;
                let length = state.get(SystemState::DebugTextLength) as u32;
                if let Some(s) = self.read_string(offset, length) {
                    sim.emit_debug_text(self.handle, s);
                }
            }

            self.write_system_state(&state);
        }
        Ok(())
    }

    fn write_target(&mut self, target: Vec2) {
        let mut state = self.read_system_state();
        state.set(SystemState::RadarContactPositionX, target.x);
        state.set(SystemState::RadarContactPositionY, target.y);
        self.write_system_state(&state);
    }
}

struct LocalSystemState {
    pub state: [f64; SystemState::Size as usize],
}

impl LocalSystemState {
    pub fn get(&self, index: SystemState) -> f64 {
        self.state[index as usize]
    }

    pub fn set(&mut self, index: SystemState, value: f64) {
        self.state[index as usize] = value;
    }
}

fn translate_class(class: ShipClass) -> Class {
    match class {
        ShipClass::Fighter => Class::Fighter,
        ShipClass::Frigate => Class::Frigate,
        ShipClass::Cruiser => Class::Cruiser,
        ShipClass::Asteroid { .. } => Class::Asteroid,
        ShipClass::Target => Class::Target,
        ShipClass::Missile => Class::Missile,
        ShipClass::Torpedo => Class::Torpedo,
    }
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
