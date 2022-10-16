pub mod builtin;

use crate::debug;
use crate::ship::{ShipClass, ShipHandle};
use crate::simulation::{Code, Simulation};
use nalgebra::point;
use nalgebra::Vector2;
use oort_api::{Ability, Class, Line, SystemState};
use serde::{Deserialize, Serialize};
use wasmer::{imports, Instance, Module, Store, WasmPtr};

pub type Vec2 = nalgebra::Vector2<f64>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub line: usize,
    pub msg: String,
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(err: wasm_bindgen::JsValue) -> Self {
        Self {
            line: 0,
            msg: format!("JS error: {:?}", err),
        }
    }
}

impl From<wasmer::InstantiationError> for Error {
    fn from(err: wasmer::InstantiationError) -> Self {
        Self {
            line: 0,
            msg: format!("Wasmer instantiation error: {:?}", err),
        }
    }
}

pub trait TeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: &mut Simulation,
    ) -> Result<Box<dyn ShipController>, Error>;
}

pub trait ShipController {
    fn tick(&mut self) -> Result<(), Error>;
    fn delete(&mut self);
    fn write_target(&mut self, target: Vector2<f64>);
}

pub fn new_team_controller(code: &Code) -> Result<Box<dyn TeamController>, Error> {
    match code {
        Code::Wasm(b) => WasmTeamController::create(b),
        Code::Builtin(name) => match builtin::load_compiled(name) {
            Ok(code) => new_team_controller(&code),
            Err(e) => Err(Error { line: 0, msg: e }),
        },
        _ => unreachable!(),
    }
}

pub struct WasmTeamController {
    shared: WasmShared,
}

struct WasmShipController {
    sim: *mut Simulation,
    handle: ShipHandle,
    shared: WasmShared,
    state: LocalSystemState,
}

#[derive(Clone)]
pub struct WasmShared {
    memory: wasmer::Memory,
    system_state_ptr: WasmPtr<f64>,
    tick_ship: wasmer::Function,
    delete_ship: wasmer::Function,
}

impl WasmTeamController {
    pub fn create(code: &[u8]) -> Result<Box<dyn TeamController>, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        let store = Store::new_with_engine(
            &wasmer_compiler::Universal::new(wasmer_compiler_cranelift::Cranelift::default())
                .engine(),
        );
        #[cfg(target_arch = "wasm32")]
        let store = Store::default();
        let module = translate_error(Module::new(&store, code))?;
        let import_object = imports! {};
        let instance = Instance::new(&module, &import_object)?;

        let memory = translate_error(instance.exports.get_memory("memory"))?.clone();
        let system_state_offset: i32 =
            translate_error(instance.exports.get_global("SYSTEM_STATE"))?
                .get()
                .i32()
                .unwrap();
        let system_state_ptr: WasmPtr<f64> = WasmPtr::new(system_state_offset as u32);

        let tick_ship = translate_error(instance.exports.get_function("export_tick_ship"))?.clone();
        let delete_ship =
            translate_error(instance.exports.get_function("export_delete_ship"))?.clone();

        let shared = WasmShared {
            memory,
            system_state_ptr,
            tick_ship,
            delete_ship,
        };

        Ok(Box::new(WasmTeamController { shared }))
    }
}

impl TeamController for WasmTeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: &mut Simulation,
    ) -> Result<Box<dyn ShipController>, Error> {
        let mut ctrl = WasmShipController {
            handle,
            sim,
            shared: self.shared.clone(),
            state: LocalSystemState::new(),
        };

        ctrl.state.set(
            SystemState::Seed,
            (make_seed(sim.seed(), handle) & 0xffffff) as f64,
        );
        if let Some(radar) = sim.ship(handle).data().radar.as_ref() {
            ctrl.state.set(SystemState::RadarHeading, radar.heading);
            ctrl.state.set(SystemState::RadarWidth, radar.width);
            ctrl.state
                .set(SystemState::RadarMinDistance, radar.min_distance);
            ctrl.state
                .set(SystemState::RadarMaxDistance, radar.max_distance);
        }

        Ok(Box::new(ctrl))
    }
}

impl WasmShipController {
    pub fn read_system_state(&mut self) {
        let slice = self
            .shared
            .system_state_ptr
            .slice(&self.shared.memory, SystemState::Size as u32)
            .expect("system state read");
        slice
            .read_slice(&mut self.state.state)
            .expect("system state read");
    }

    pub fn write_system_state(&self) {
        let slice = self
            .shared
            .system_state_ptr
            .slice(&self.shared.memory, SystemState::Size as u32)
            .expect("system state write");
        slice
            .write_slice(&self.state.state)
            .expect("system state write");
    }

    pub fn read_string(&self, offset: u32, length: u32) -> Option<String> {
        let ptr: WasmPtr<u8> = WasmPtr::new(offset);
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(length as usize, 0);
        let slice = ptr.slice(&self.shared.memory, length).ok()?;
        slice.read_slice(&mut bytes).ok()?;
        String::from_utf8(bytes).ok()
    }

    pub fn read_vec<T: Default + Clone>(&self, offset: u32, length: u32) -> Option<Vec<T>> {
        let ptr: WasmPtr<u8> = WasmPtr::new(offset);
        let byte_length = length.saturating_mul(std::mem::size_of::<T>() as u32);
        let slice = ptr.slice(&self.shared.memory, byte_length).ok()?;
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
    fn tick(&mut self) -> Result<(), Error> {
        {
            let sim = unsafe { &mut *self.sim };

            self.state.set(
                SystemState::Class,
                translate_class(sim.ship(self.handle).data().class) as u32 as f64,
            );

            let position = sim.ship(self.handle).position();
            self.state.set(SystemState::PositionX, position.x);
            self.state.set(SystemState::PositionY, position.y);

            let velocity = sim.ship(self.handle).velocity();
            self.state.set(SystemState::VelocityX, velocity.x);
            self.state.set(SystemState::VelocityY, velocity.y);

            self.state
                .set(SystemState::Heading, sim.ship(self.handle).heading());
            self.state.set(
                SystemState::AngularVelocity,
                sim.ship(self.handle).angular_velocity(),
            );

            if let Some(radar) = sim.ship_mut(self.handle).data_mut().radar.as_mut() {
                self.state
                    .set(SystemState::RadarHeading, radar.get_heading());
                self.state.set(SystemState::RadarWidth, radar.get_width());
                self.state
                    .set(SystemState::RadarMinDistance, radar.get_min_distance());
                self.state
                    .set(SystemState::RadarMaxDistance, radar.get_max_distance());

                if let Some(contact) = radar.scan() {
                    self.state.set(SystemState::RadarContactFound, 1.0);
                    self.state
                        .set(SystemState::RadarContactPositionX, contact.position.x);
                    self.state
                        .set(SystemState::RadarContactPositionY, contact.position.y);
                    self.state
                        .set(SystemState::RadarContactVelocityX, contact.velocity.x);
                    self.state
                        .set(SystemState::RadarContactVelocityY, contact.velocity.y);
                    self.state.set(
                        SystemState::RadarContactClass,
                        translate_class(contact.class) as u32 as f64,
                    );
                } else {
                    self.state.set(SystemState::RadarContactFound, 0.0);
                }
            }

            {
                let ship = sim.ship(self.handle);
                let data = ship.data();
                let max_acceleration = data.max_acceleration;
                self.state
                    .set(SystemState::MaxAccelerationX, max_acceleration.x);
                self.state
                    .set(SystemState::MaxAccelerationY, max_acceleration.y);
                self.state.set(
                    SystemState::MaxAngularAcceleration,
                    data.max_angular_acceleration,
                );
            }

            if let Some(radio) = sim.ship(self.handle).data().radio.as_ref() {
                self.state
                    .set(SystemState::RadioChannel, radio.get_channel() as f64);
                if let Some(msg) = radio.get_received() {
                    self.state.set(SystemState::RadioReceive, 1.0);
                    self.state.set(SystemState::RadioData0, msg[0]);
                    self.state.set(SystemState::RadioData1, msg[1]);
                    self.state.set(SystemState::RadioData2, msg[2]);
                    self.state.set(SystemState::RadioData3, msg[3]);
                }
                self.state.set(SystemState::RadioSend, 0.0);
            }

            self.state.set(SystemState::CurrentTick, sim.tick() as f64);
            self.state
                .set(SystemState::Energy, sim.ship(self.handle).data().energy);

            self.write_system_state();
        }

        let (index, _) = self.handle.0.into_raw_parts();
        let index = index as i32;
        translate_error(self.shared.tick_ship.call(&[index.into()]))?;

        {
            self.read_system_state();
            let sim = unsafe { &mut *self.sim };

            sim.ship_mut(self.handle).accelerate(Vec2::new(
                self.state.get(SystemState::AccelerateX),
                self.state.get(SystemState::AccelerateY),
            ));
            self.state.set(SystemState::AccelerateX, 0.0);
            self.state.set(SystemState::AccelerateY, 0.0);

            sim.ship_mut(self.handle)
                .torque(self.state.get(SystemState::Torque));
            self.state.set(SystemState::Torque, 0.0);

            for (i, (aim, fire)) in [
                (SystemState::Aim0, SystemState::Fire0),
                (SystemState::Aim1, SystemState::Fire1),
                (SystemState::Aim2, SystemState::Fire2),
                (SystemState::Aim3, SystemState::Fire3),
            ]
            .iter()
            .enumerate()
            {
                if self.state.get(*fire) > 0.0 {
                    sim.ship_mut(self.handle)
                        .aim(i as i64, self.state.get(*aim));
                    sim.ship_mut(self.handle).fire(i as i64);
                    self.state.set(*fire, 0.0);
                }
            }

            if let Some(radar) = sim.ship_mut(self.handle).data_mut().radar.as_mut() {
                radar.set_heading(self.state.get(SystemState::RadarHeading));
                radar.set_width(self.state.get(SystemState::RadarWidth));
                radar.set_min_distance(self.state.get(SystemState::RadarMinDistance));
                radar.set_max_distance(self.state.get(SystemState::RadarMaxDistance));
            }

            if let Some(ability) = translate_ability(self.state.get(SystemState::ActivateAbility)) {
                if ability != Ability::None {
                    sim.ship_mut(self.handle).activate_ability(ability);
                }
            }

            if self.state.get(SystemState::Explode) > 0.0 {
                sim.ship_mut(self.handle).explode();
                self.state.set(SystemState::Explode, 0.0);
            }

            if self.state.get(SystemState::DebugTextLength) > 0.0 {
                let offset = self.state.get(SystemState::DebugTextPointer) as u32;
                let length = self.state.get(SystemState::DebugTextLength) as u32;
                if let Some(s) = self.read_string(offset, length) {
                    sim.emit_debug_text(self.handle, s);
                }
            }

            if self.state.get(SystemState::DebugLinesLength) > 0.0 {
                let offset = self.state.get(SystemState::DebugLinesPointer) as u32;
                let length = self.state.get(SystemState::DebugLinesLength) as u32;
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
                radio.set_channel(self.state.get(SystemState::RadioChannel) as usize);
                if self.state.get(SystemState::RadioSend) != 0.0 {
                    let msg = [
                        self.state.get(SystemState::RadioData0),
                        self.state.get(SystemState::RadioData1),
                        self.state.get(SystemState::RadioData2),
                        self.state.get(SystemState::RadioData3),
                    ];
                    radio.set_sent(Some(msg));
                }
            }
        }
        Ok(())
    }

    fn delete(&mut self) {
        let (index, _) = self.handle.0.into_raw_parts();
        let index = index as i32;
        if let Err(e) = translate_error(self.shared.delete_ship.call(&[index.into()])) {
            log::warn!("Failed to delete ship: {:?}", e);
        }
    }

    fn write_target(&mut self, target: Vec2) {
        self.state.set(SystemState::RadarContactPositionX, target.x);
        self.state.set(SystemState::RadarContactPositionY, target.y);
    }
}

struct LocalSystemState {
    pub state: [f64; SystemState::Size as usize],
}

impl LocalSystemState {
    pub fn new() -> Self {
        Self {
            state: [0.0; SystemState::Size as usize],
        }
    }

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

fn translate_ability(v: f64) -> Option<Ability> {
    let v = v as u32;
    if v == Ability::None as u32 {
        Some(Ability::None)
    } else if v == Ability::Boost as u32 {
        Some(Ability::Boost)
    } else if v == Ability::ShapedCharge as u32 {
        Some(Ability::ShapedCharge)
    } else {
        None
    }
}

fn translate_error<T, U>(err: Result<T, U>) -> Result<T, Error>
where
    U: std::fmt::Debug,
{
    match err {
        Ok(val) => Ok(val),
        Err(err) => Err(Error {
            line: 0,
            msg: format!("Wasmer error: {:?}", err),
        }),
    }
}
