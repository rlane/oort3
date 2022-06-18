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

const WASM: &[u8] = include_bytes!("../../../../ai/reference.wasm");

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
    pub fn read_system_state(&self) -> LocalSystemState {
        let mut state = [0.0; SystemState::Size as usize];
        let mut ptr = self.system_state_ptr;
        for i in 0..SystemState::Size as usize {
            state[i] = ptr.deref(&self.memory).unwrap().get();
            ptr = WasmPtr::new(ptr.offset() + 8);
        }
        LocalSystemState { state }
    }

    pub fn write_system_state(&self, state: &LocalSystemState) {
        let mut ptr = self.system_state_ptr;
        for i in 0..SystemState::Size as usize {
            ptr.deref(&self.memory).unwrap().set(state.state[i]);
            ptr = WasmPtr::new(ptr.offset() + 8);
        }
    }
}

impl ShipController for WasmShipController {
    fn tick(&mut self) -> Result<(), super::Error> {
        translate_error(self.tick.call(&[]))?;

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

        for (i, launch) in [
            SystemState::Missile0Launch,
            SystemState::Missile1Launch,
            SystemState::Missile2Launch,
            SystemState::Missile3Launch,
        ]
        .iter()
        .enumerate()
        {
            if state.get(*launch) > 0.0 {
                sim.ship_mut(self.handle)
                    .launch_missile(i as i64, "".to_string());
                state.set(*launch, 0.0);
            }
        }

        if state.get(SystemState::Explode) > 0.0 {
            sim.ship_mut(self.handle).explode();
            state.set(SystemState::Explode, 0.0);
        }

        if let Some(radar) = sim.ship_mut(self.handle).data_mut().radar.as_mut() {
            radar.heading = state.get(SystemState::RadarHeading);
            radar.width = state.get(SystemState::RadarWidth);
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
        Ok(())
    }

    fn write_target(&mut self, _target: Vec2) {}
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
