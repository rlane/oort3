use crate::bullet::{self, BulletData, BulletHandle};
use crate::collision;
use crate::debug;
pub use crate::debug::Line;
use crate::index_set::{HasIndex, IndexSet};
use crate::radar;
use crate::radio;
use crate::scenario;
use crate::scenario::Scenario;
use crate::ship::{ShipAccessor, ShipAccessorMut, ShipData, ShipHandle, Target};
use crate::snapshot::*;
use crate::vm;
use crate::vm::TeamController;
use crossbeam::channel::Sender;
use instant::Instant;
use nalgebra::{Vector2, Vector4};
use oort_api::Text;
use rand_chacha::ChaCha8Rng;
use rapier2d_f64::data::Coarena;
use rapier2d_f64::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub const WORLD_SIZE: f64 = 20000.0;
pub const PHYSICS_TICK_LENGTH: f64 = 1.0 / 60.0;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum Code {
    None,
    Rust(String),
    Wasm(Vec<u8>),
    Builtin(String),
    #[cfg(feature = "precompile")]
    Precompiled(bytes::Bytes),
}

pub struct Simulation {
    scenario: Option<Box<dyn Scenario>>,
    pub ships: IndexSet<ShipHandle>,
    pub(crate) ship_data: Coarena<ShipData>,
    team_controllers: HashMap<i32, Rc<RefCell<Box<TeamController>>>>,
    pub new_ships: Vec<(/*team*/ i32, ShipHandle)>,
    pub bullets: IndexSet<BulletHandle>,
    pub(crate) bullet_data: Coarena<BulletData>,
    pub(crate) bodies: RigidBodySet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) colliders: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    event_collector: CollisionEventHandler,
    contact_recv: crossbeam::channel::Receiver<CollisionEvent>,
    pub(crate) events: SimEvents,
    tick: u32,
    pub cheats: bool,
    seed: u32,
    timing: Timing,
    pub(crate) rng: ChaCha8Rng,
}

impl Simulation {
    pub fn new(scenario_name: &str, seed: u32, codes: &[Code]) -> Box<Simulation> {
        log::info!("seed {seed}");
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let mut sim = Box::new(Simulation {
            scenario: None,
            ships: IndexSet::new(),
            ship_data: Coarena::new(),
            team_controllers: HashMap::new(),
            new_ships: Vec::new(),
            bullets: IndexSet::new(),
            bullet_data: Coarena::new(),
            bodies: RigidBodySet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            colliders: ColliderSet::new(),
            integration_parameters: IntegrationParameters {
                dt: PHYSICS_TICK_LENGTH,
                max_ccd_substeps: 2,
                ..Default::default()
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            event_collector: CollisionEventHandler::new(contact_send),
            contact_recv,
            events: SimEvents::new(),
            tick: 0,
            cheats: false,
            seed,
            timing: Default::default(),
            rng: crate::rng::new_rng(seed),
        });

        for (team, code) in codes.iter().enumerate() {
            if !matches!(code, Code::None) {
                sim.upload_code(team as i32, code);
            }
        }

        let mut scenario = scenario::load(scenario_name);
        scenario.init(&mut sim, seed);
        sim.scenario = Some(scenario);

        sim
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn time(&self) -> f64 {
        self.tick as f64 * PHYSICS_TICK_LENGTH
    }

    pub fn score_time(&self) -> f64 {
        self.scenario.as_ref().unwrap().score_time(self)
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn status(&self) -> scenario::Status {
        self.scenario.as_ref().unwrap().status(self)
    }

    pub fn ship(self: &Simulation, handle: ShipHandle) -> ShipAccessor {
        ShipAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn ship_mut(self: &mut Simulation, handle: ShipHandle) -> ShipAccessorMut {
        ShipAccessorMut {
            simulation: self,
            handle,
        }
    }

    #[allow(clippy::let_unit_value)]
    pub fn step(self: &mut Simulation) {
        self.events.clear();

        let new_ships = std::mem::take(&mut self.new_ships);
        for (team, handle) in new_ships.iter() {
            if let Some(team_ctrl) = self.get_team_controller(*team) {
                if let Err(e) = team_ctrl.borrow_mut().add_ship(*handle, self) {
                    log::warn!("Ship creation error: {:?}", e);
                    self.events.errors.push(e);
                }
            }
        }

        let physics_start_time = Instant::now();
        let gravity = vector![0.0, 0.0];
        let physics_hooks = ();
        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &self.event_collector,
        );

        let collision_events: Vec<_> = self.contact_recv.try_iter().collect();
        collision::handle_collisions(self, &collision_events);
        radar::tick(self);
        radio::tick(self);

        self.timing.physics = (Instant::now() - physics_start_time).as_secs_f64();

        let script_start_time = Instant::now();
        let mut teams: Vec<_> = self
            .team_controllers
            .iter()
            .map(|(k, v)| (*k, Rc::clone(v)))
            .collect();
        teams.sort_by_key(|(k, _)| *k);

        for (_, team_controller) in teams.iter() {
            team_controller.borrow_mut().tick(self);
        }

        let handle_snapshot: Vec<ShipHandle> = self.ships.iter().cloned().collect();
        for handle in handle_snapshot {
            debug::emit_ship(self, handle);
            self.ship_mut(handle).tick();
        }
        self.timing.script = (Instant::now() - script_start_time).as_secs_f64();

        let physics_start_time = Instant::now();
        bullet::tick(self);
        self.timing.physics += (Instant::now() - physics_start_time).as_secs_f64();

        let mut scenario = std::mem::take(&mut self.scenario);
        scenario.as_mut().unwrap().tick(self);
        self.scenario = scenario;

        self.tick += 1;
    }

    pub fn upload_code(&mut self, team: i32, code: &Code) {
        match vm::new_team_controller(code) {
            Ok(team_ctrl) => {
                self.team_controllers
                    .insert(team, Rc::new(RefCell::new(team_ctrl)));
            }
            Err(e) => {
                log::warn!("Creating team controller failed: {:?}", e);
                self.events.errors.push(e);
            }
        }
    }

    pub fn events(&self) -> &SimEvents {
        &self.events
    }

    pub fn emit_debug_lines(&mut self, ship: ShipHandle, lines: Vec<Line>) {
        self.events.debug_lines.push((ship.into(), lines));
    }

    pub fn emit_debug_text(&mut self, ship: ShipHandle, s: String) {
        self.events.debug_text.insert(ship.into(), s);
    }

    pub fn emit_drawn_text(&mut self, ship: ShipHandle, texts: &[Text]) {
        self.events
            .drawn_text
            .entry(ship.into())
            .or_default()
            .extend(texts.iter().cloned());
    }

    pub fn write_target(&mut self, ship: ShipHandle, p: Vector2<f64>, v: Vector2<f64>) {
        self.ship_mut(ship).data_mut().target = Some(Box::new(Target {
            position: p,
            velocity: v,
        }));
    }

    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let fixedpoint = |v: f64| (v * 1e9) as i64;
        let mut s = DefaultHasher::new();
        for handle in self.ships.iter() {
            let ship = self.ship(*handle);
            s.write_i64(fixedpoint(ship.position().x));
            s.write_i64(fixedpoint(ship.position().y));
            s.write_i64(fixedpoint(ship.heading()));
            s.write_i64(fixedpoint(ship.velocity().x));
            s.write_i64(fixedpoint(ship.velocity().y));
            s.write_i64(fixedpoint(ship.angular_velocity()));
            s.write_i64(fixedpoint(ship.data().health));
        }
        for handle in self.bullets.iter() {
            let body = bullet::body(self, *handle);
            s.write_i64(fixedpoint(body.translation().x));
            s.write_i64(fixedpoint(body.translation().y));
        }
        s.finish()
    }

    pub fn snapshot(&self, nonce: u32) -> Snapshot {
        let mut snapshot = Snapshot {
            nonce,
            time: self.time(),
            score_time: self.score_time(),
            status: self.status(),
            ships: vec![],
            bullets: vec![],
            scenario_lines: self.scenario.as_ref().unwrap().lines(),
            debug_lines: self.events.debug_lines.clone(),
            debug_text: self.events.debug_text.clone(),
            drawn_text: self.events.drawn_text.clone(),
            particles: self.events.particles.clone(),
            errors: self.events.errors.clone(),
            cheats: self.cheats,
            timing: self.timing.clone(),
        };

        for &handle in self.ships.iter() {
            let ship = self.ship(handle);
            let id = handle.into();
            let position = ship.position().vector.into();
            let team = ship.data().team;
            let class = ship.data().class;
            let health = ship.data().health;
            snapshot.ships.push(ShipSnapshot {
                id,
                position,
                velocity: ship.velocity(),
                heading: ship.heading(),
                angular_velocity: ship.angular_velocity(),
                team,
                class,
                health,
                active_abilities: ship.active_abilities(),
            });
        }

        for &handle in self.bullets.iter() {
            let body = self.bodies.get(handle.into()).unwrap();
            let data = self.bullet_data.get(handle.index()).unwrap();
            snapshot.bullets.push(BulletSnapshot {
                position: body.position().translation.vector.into(),
                velocity: *body.linvel(),
                color: data.color,
                ttl: data.ttl,
            });
        }

        snapshot
    }

    pub fn get_team_controller(&mut self, team: i32) -> Option<Rc<RefCell<Box<TeamController>>>> {
        self.team_controllers.get_mut(&team).map(|x| x.clone())
    }
}

pub struct CollisionEventHandler {
    collision_event_sender: Sender<CollisionEvent>,
}

impl CollisionEventHandler {
    pub fn new(collision_event_sender: Sender<CollisionEvent>) -> CollisionEventHandler {
        CollisionEventHandler {
            collision_event_sender,
        }
    }
}

impl EventHandler for CollisionEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&rapier2d_f64::geometry::ContactPair>,
    ) {
        let _ = self.collision_event_sender.send(event);
    }

    fn handle_contact_force_event(
        &self,
        _: f64,
        _: &rapier2d_f64::dynamics::RigidBodySet,
        _: &rapier2d_f64::geometry::ColliderSet,
        _: &ContactPair,
        _: f64,
    ) {
        unimplemented!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub color: Vector4<f32>,
    pub lifetime: f32,
}

pub struct SimEvents {
    pub errors: Vec<vm::Error>,
    pub particles: Vec<Particle>,
    pub debug_lines: Vec<(u64, Vec<Line>)>,
    pub debug_text: BTreeMap<u64, String>,
    pub drawn_text: BTreeMap<u64, Vec<Text>>,
}

impl SimEvents {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            particles: vec![],
            debug_lines: Vec::new(),
            debug_text: BTreeMap::new(),
            drawn_text: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.particles.clear();
        self.debug_lines.clear();
        self.debug_text.clear();
        self.drawn_text.clear();
    }
}

impl Default for SimEvents {
    fn default() -> Self {
        SimEvents::new()
    }
}
