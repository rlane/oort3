use super::{ShipController, TeamController};
use crate::debug;
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::Simulation;
use nalgebra::point;
use oort_shared::*;
use wasmer::{imports, Instance, Module, Store, WasmPtr};

pub type Vec2 = nalgebra::Vector2<f64>;

pub struct WasmTeamController {
    pub module: Module,
}

impl WasmTeamController {
    pub fn create(code: &[u8]) -> Result<Box<dyn TeamController>, super::Error> {
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
            state.set(SystemState::RadarMinDistance, radar.min_distance);
            state.set(SystemState::RadarMaxDistance, radar.max_distance);
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

    pub fn read_vec<T: Default + Clone>(&self, offset: u32, length: u32) -> Option<Vec<T>> {
        let ptr: WasmPtr<u8> = WasmPtr::new(offset);
        let byte_length = length.saturating_mul(std::mem::size_of::<T>() as u32);
        let slice = ptr.slice(&self.memory, byte_length).ok()?;
        let byte_vec = slice.read_to_vec().ok()?;
        let src_ptr = unsafe { std::mem::transmute::<*const u8, *const T>(byte_vec.as_ptr()) };
        let src_slice = unsafe { std::slice::from_raw_parts(src_ptr, length as usize) };
        Some(src_slice.to_vec())
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

            if let Some(radar) = sim.ship_mut(self.handle).data_mut().radar.as_mut() {
                state.set(SystemState::RadarHeading, radar.get_heading());
                state.set(SystemState::RadarWidth, radar.get_width());
                state.set(SystemState::RadarMinDistance, radar.get_min_distance());
                state.set(SystemState::RadarMaxDistance, radar.get_max_distance());

                if let Some(contact) = radar.scan() {
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
            }

            {
                let ship = sim.ship(self.handle);
                let data = ship.data();
                state.set(SystemState::MaxAccelerationX, data.max_acceleration.x);
                state.set(SystemState::MaxAccelerationY, data.max_acceleration.y);
                state.set(
                    SystemState::MaxAngularAcceleration,
                    data.max_angular_acceleration,
                );
            }

            if let Some(radio) = sim.ship(self.handle).data().radio.as_ref() {
                state.set(SystemState::RadioChannel, radio.get_channel() as f64);
                state.set(
                    SystemState::RadioReceive,
                    radio.get_received().unwrap_or(0.0),
                );
                state.set(SystemState::RadioSend, 0.0);
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
                radar.set_heading(state.get(SystemState::RadarHeading));
                radar.set_width(state.get(SystemState::RadarWidth));
                radar.set_min_distance(state.get(SystemState::RadarMinDistance));
                radar.set_max_distance(state.get(SystemState::RadarMaxDistance));
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

            if state.get(SystemState::DebugLinesLength) > 0.0 {
                let offset = state.get(SystemState::DebugLinesPointer) as u32;
                let length = state.get(SystemState::DebugLinesLength) as u32;
                if length <= 128 {
                    if let Some(lines) = self.read_vec::<Line>(offset, length) {
                        sim.emit_debug_lines(
                            self.handle,
                            &lines
                                .iter()
                                .map(|v| crate::debug::Line {
                                    a: point![v.x0, v.y0],
                                    b: point![v.x1, v.y1],
                                    color: debug::convert_color(v.color),
                                })
                                .collect::<Vec<debug::Line>>(),
                        );
                    }
                }
            }

            if let Some(radio) = sim.ship_mut(self.handle).data_mut().radio.as_mut() {
                radio.set_channel(state.get(SystemState::RadioChannel) as usize);
                let data = state.get(SystemState::RadioSend);
                if data != 0.0 {
                    radio.set_sent(Some(data));
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
        let v = self.state[index as usize];
        if v.is_nan() || v.is_infinite() {
            0.0
        } else {
            v
        }
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
