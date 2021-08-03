pub mod bullet;
pub mod collision;
pub mod debug;
pub mod index_set;
pub mod rng;
pub mod scenario;
pub mod ship;
pub mod snapshot;

use self::bullet::{BulletAccessor, BulletAccessorMut, BulletData, BulletHandle};
use self::index_set::IndexSet;
use self::scenario::Scenario;
use self::ship::{ShipAccessor, ShipAccessorMut, ShipData, ShipHandle};
use crate::script;
use crate::script::{ShipController, TeamController};
use crossbeam::channel::Sender;
pub use debug::Line;
use nalgebra::Vector2;
use rapier2d_f64::prelude::*;
use snapshot::*;
use std::collections::HashMap;

pub const WORLD_SIZE: f64 = 10000.0;
pub(crate) const PHYSICS_TICK_LENGTH: f64 = 1.0 / 60.0;

pub struct Simulation {
    scenario: Option<Box<dyn Scenario>>,
    pub ships: IndexSet<ShipHandle>,
    ship_data: HashMap<ShipHandle, ShipData>,
    team_controllers: HashMap<i32, Box<dyn TeamController>>,
    ship_controllers: HashMap<ShipHandle, Box<dyn ShipController>>,
    pub bullets: IndexSet<BulletHandle>,
    bullet_data: HashMap<BulletHandle, BulletData>,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    event_collector: CollisionEventHandler,
    contact_recv: crossbeam::channel::Receiver<ContactEvent>,
    events: SimEvents,
    tick: u32,
    pub cheats: bool,
}

impl Simulation {
    pub fn new(scenario_name: &str, seed: u64, mut code: &str) -> Box<Simulation> {
        if code.is_empty() {
            code = "fn tick(){}";
        }
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let mut sim = Box::new(Simulation {
            scenario: None,
            ships: IndexSet::new(),
            ship_data: HashMap::new(),
            team_controllers: HashMap::new(),
            ship_controllers: HashMap::new(),
            bullets: IndexSet::new(),
            bullet_data: HashMap::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
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
        });

        sim.upload_code(/*team=*/ 0, code);

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

    pub fn bullet(self: &Simulation, handle: BulletHandle) -> BulletAccessor {
        BulletAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn bullet_mut(self: &mut Simulation, handle: BulletHandle) -> BulletAccessorMut {
        BulletAccessorMut {
            simulation: self,
            handle,
        }
    }

    pub fn step(self: &mut Simulation) {
        self.events.clear();
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
            &mut self.joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &self.event_collector,
        );

        while let Ok(event) = self.contact_recv.try_recv() {
            if let ContactEvent::Started(h1, h2) = event {
                let get_index = |h| self.colliders.get(h).and_then(|x| x.parent()).map(|x| x.0);
                let handle_hit = |sim: &mut Simulation, ship, bullet| {
                    if sim.bullet(bullet).data().team == sim.ship(ship).data().team {
                        sim.bullet_mut(bullet).destroy();
                        return;
                    }
                    let damage = sim.bullet(bullet).data().damage;
                    let ship_destroyed = {
                        let ship_data = sim.ship_data.get_mut(&ship).unwrap();
                        ship_data.health -= damage;
                        ship_data.health <= 0.0
                    };
                    if ship_destroyed {
                        sim.events
                            .ships_destroyed
                            .push(sim.ship(ship).body().position().translation.vector);
                        sim.ship_mut(ship).data_mut().destroyed = true;
                    }
                    sim.events
                        .hits
                        .push(sim.bullet(bullet).body().position().translation.vector);
                    sim.bullet_mut(bullet).destroy();
                };
                if let (Some(idx1), Some(idx2)) = (get_index(h1), get_index(h2)) {
                    if self.bullets.contains(BulletHandle(idx1)) {
                        if self.ships.contains(ShipHandle(idx2)) {
                            handle_hit(self, ShipHandle(idx2), BulletHandle(idx1));
                        } else {
                            self.bullet_mut(BulletHandle(idx1)).destroy();
                        }
                    } else if self.bullets.contains(BulletHandle(idx2)) {
                        if self.ships.contains(ShipHandle(idx1)) {
                            handle_hit(self, ShipHandle(idx1), BulletHandle(idx2));
                        } else {
                            self.bullet_mut(BulletHandle(idx2)).destroy();
                        }
                    }
                }
            }
        }

        let handle_snapshot: Vec<ShipHandle> = self.ships.iter().cloned().collect();
        for handle in handle_snapshot {
            if let Some(ship_controller) = self.ship_controllers.get_mut(&handle) {
                if let Err(e) = ship_controller.tick() {
                    self.events.errors.push(e);
                }
            }
            debug::emit_ship(self, handle);
            self.ship_mut(handle).tick();
        }

        let mut scenario = std::mem::take(&mut self.scenario);
        scenario.as_mut().unwrap().tick(self);
        self.scenario = scenario;

        self.tick += 1;
    }

    pub fn upload_code(&mut self, team: i32, code: &str) {
        match script::new_team_controller(code) {
            Ok(team_ctrl) => {
                self.team_controllers.insert(team, team_ctrl);
            }
            Err(e) => {
                self.events.errors.push(e);
            }
        }
    }

    pub fn events(&self) -> &SimEvents {
        &self.events
    }

    pub fn emit_debug_lines(&mut self, lines: &[Line]) {
        self.events.debug_lines.extend(lines.iter().cloned());
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
        s.finish()
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = Snapshot {
            time: self.time(),
            status: self.status(),
            ships: vec![],
            bullets: vec![],
            scenario_lines: self.scenario.as_ref().unwrap().lines(),
            debug_lines: self.events.debug_lines.clone(),
            hits: self.events.hits.clone(),
            ships_destroyed: self.events.ships_destroyed.clone(),
            errors: self.events.errors.clone(),
            cheats: self.cheats,
        };

        for &handle in self.ships.iter() {
            let ship = self.ship(handle);
            let (gen, idx) = handle.0.into_raw_parts();
            let id = ((gen as u64) << 32) | idx as u64;
            let position = ship.position().vector.into();
            let team = ship.data().team;
            let class = ship.data().class;
            snapshot.ships.push(ShipSnapshot {
                id,
                position,
                heading: ship.heading(),
                team,
                class,
            });
        }

        for &handle in self.bullets.iter() {
            let bullet = self.bullet(handle);
            snapshot.bullets.push(BulletSnapshot {
                position: bullet.body().position().translation.vector.into(),
                velocity: *bullet.body().linvel(),
            });
        }

        snapshot
    }
}

pub struct CollisionEventHandler {
    contact_event_sender: Sender<ContactEvent>,
}

impl CollisionEventHandler {
    pub fn new(contact_event_sender: Sender<ContactEvent>) -> CollisionEventHandler {
        CollisionEventHandler {
            contact_event_sender,
        }
    }
}

impl EventHandler for CollisionEventHandler {
    fn handle_intersection_event(&self, _event: IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent, _contact_pair: &ContactPair) {
        let _ = self.contact_event_sender.send(event);
    }
}

pub struct SimEvents {
    pub errors: Vec<script::Error>,
    pub hits: Vec<Vector2<f64>>,
    pub ships_destroyed: Vec<Vector2<f64>>,
    pub debug_lines: Vec<Line>,
}

impl SimEvents {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            hits: vec![],
            ships_destroyed: vec![],
            debug_lines: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.hits.clear();
        self.ships_destroyed.clear();
        self.debug_lines.clear();
    }
}

impl Default for SimEvents {
    fn default() -> Self {
        SimEvents::new()
    }
}
