use crate::color;
use crate::debug;
use crate::ship::ShipHandle;
use crate::simulation::{NativeShipFactory, NativeShip, Simulation};
use nalgebra::point;
use oort_api::SystemState;
use std::collections::HashMap;

use super::{
    apply_system_state, generate_system_state, make_seed, validate_lines, validate_texts,
    Environment, Error, LocalSystemState, TeamControllerTrait, MAX_DEBUG_LINES, MAX_DRAWN_TEXT,
};

struct NativeShipController {
    ship: Box<dyn NativeShip>,
    state: LocalSystemState,
}

pub struct NativeTeamController {
    factory: NativeShipFactory,
    ships: HashMap<ShipHandle, NativeShipController>,
    next_id: u32,
    environment: Environment,
}

impl NativeTeamController {
    pub fn new(factory: NativeShipFactory) -> Self {
        Self {
            factory,
            ships: HashMap::new(),
            next_id: 1,
            environment: Environment::new(),
        }
    }
}

impl TeamControllerTrait for NativeTeamController {
    fn add_ship(&mut self, handle: ShipHandle, sim: &Simulation) -> Result<(), Error> {
        let mut state = LocalSystemState::new();
        state.set(
            SystemState::Seed,
            (make_seed(sim.seed(), handle) & 0xffffff) as f64,
        );
        state.set(SystemState::Id, self.next_id as f64);
        self.next_id += 1;

        for (idx, radar) in sim.ship(handle).data().radars.iter().enumerate() {
            let idxs = oort_api::prelude::radar_internal::radar_control_indices(idx);
            state.set(idxs.heading, radar.heading);
            state.set(idxs.width, radar.width);
            state.set(idxs.min_distance, radar.min_distance);
            state.set(idxs.max_distance, radar.max_distance);
        }

        // Write state into the global SYSTEM_STATE so Ship::new() can read seed etc.
        write_state_to_global(&state);

        // Write environment
        write_environment_to_global(&self.environment);

        // Initialize RNG and create ship
        unsafe {
            oort_api::rng_state::set(oort_api::rng_state::RngState::new());
        }
        let ship = (self.factory)();

        self.ships.insert(
            handle,
            NativeShipController { ship, state },
        );
        Ok(())
    }

    fn remove_ship(&mut self, handle: ShipHandle) {
        self.ships.remove(&handle);
    }

    fn tick(&mut self, sim: &mut Simulation) {
        let mut handles: Vec<_> = self.ships.keys().cloned().collect();
        handles.sort_by_key(|x| x.0);

        for handle in handles {
            if let Err(e) = self.tick_ship(sim, handle) {
                log::warn!("{}", e.msg);
                sim.emit_debug_text(handle, format!("Crashed: {}", e.msg.clone()));
                sim.ship_mut(handle).data_mut().crash_message = Some(e.msg);
            }
        }
    }

    fn update_environment(&mut self, environment: &Environment) -> Result<(), Error> {
        self.environment = environment.clone();
        Ok(())
    }
}

impl NativeTeamController {
    fn tick_ship(&mut self, sim: &mut Simulation, handle: ShipHandle) -> Result<(), Error> {
        // Skip crashed ships
        if sim.ship(handle).data().crash_message.is_some() {
            return Ok(());
        }

        let ship_ctrl = self.ships.get_mut(&handle).unwrap();

        // Fill LocalSystemState from simulation
        generate_system_state(sim, handle, &mut ship_ctrl.state);

        // Write to the global SYSTEM_STATE (shared process memory)
        write_state_to_global(&ship_ctrl.state);

        // Reset debug buffers, run tick, flush debug
        oort_api::dbg::reset();
        ship_ctrl.ship.tick();
        oort_api::dbg::update();

        // Read back from global SYSTEM_STATE
        read_state_from_global(&mut ship_ctrl.state);

        // Apply actions to simulation
        apply_system_state(sim, handle, &mut ship_ctrl.state);

        // Harvest debug output directly from oort_api buffers (not via SYSTEM_STATE pointers,
        // which truncate 64-bit native pointers to 32-bit).
        unsafe {
            let text = oort_api::dbg::text_buffer();
            if !text.is_empty() {
                sim.emit_debug_text(handle, text.to_string());
            }

            let lines = oort_api::dbg::line_buffer();
            if !lines.is_empty() && lines.len() <= MAX_DEBUG_LINES as usize {
                if validate_lines(lines) {
                    sim.emit_debug_lines(
                        handle,
                        lines
                            .iter()
                            .map(|v| debug::Line {
                                a: point![v.x0, v.y0],
                                b: point![v.x1, v.y1],
                                color: color::from_u24(v.color),
                            })
                            .collect(),
                    );
                }
            }

            let texts = oort_api::dbg::drawn_text_buffer();
            if !texts.is_empty() && texts.len() <= MAX_DRAWN_TEXT as usize {
                if validate_texts(texts) {
                    sim.emit_drawn_text(Some(handle), texts);
                }
            }
        }

        Ok(())
    }
}

fn write_state_to_global(state: &LocalSystemState) {
    unsafe {
        let global = &mut *std::ptr::addr_of_mut!(oort_api::sys::SYSTEM_STATE);
        let len = state.state.len();
        global[..len].copy_from_slice(&state.state);
    }
}

fn read_state_from_global(state: &mut LocalSystemState) {
    unsafe {
        let global = &*std::ptr::addr_of!(oort_api::sys::SYSTEM_STATE);
        let len = state.state.len();
        state.state.copy_from_slice(&global[..len]);
    }
}

fn write_environment_to_global(environment: &Environment) {
    let environment_string = environment
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("\n");
    if environment_string.len() <= oort_api::MAX_ENVIRONMENT_SIZE {
        unsafe {
            let global = &mut *std::ptr::addr_of_mut!(oort_api::sys::ENVIRONMENT);
            let bytes = environment_string.as_bytes();
            global[..bytes.len()].copy_from_slice(bytes);
            if bytes.len() < global.len() {
                global[bytes.len()] = 0;
            }
        }
    }
}
