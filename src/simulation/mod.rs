pub mod bullet;
pub mod index_set;
pub mod rng;
pub mod scenario;
pub mod ship;

use self::bullet::{BulletAccessor, BulletAccessorMut, BulletData, BulletHandle};
use self::index_set::IndexSet;
use self::ship::{ShipAccessor, ShipAccessorMut, ShipData, ShipHandle};
use crate::script;
use crate::script::{ShipController, TeamController};
use nalgebra::Vector2;
use nalgebra::{Point2, Vector4};
use rapier2d_f64::prelude::*;
use std::collections::HashMap;

pub const WORLD_SIZE: f64 = 10000.0;
pub(crate) const PHYSICS_TICK_LENGTH: f64 = 1.0 / 60.0;

pub(crate) const WALL_COLLISION_GROUP: u32 = 0;
pub(crate) const SHIP_COLLISION_GROUP: u32 = 1;
pub(crate) const BULLET_COLLISION_GROUP: u32 = 2;

pub struct Simulation {
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
    event_collector: rapier2d_f64::pipeline::ChannelEventCollector,
    contact_recv: crossbeam::channel::Receiver<ContactEvent>,
    intersection_recv: crossbeam::channel::Receiver<IntersectionEvent>,
    events: SimEvents,
    tick: u32,
    pub cheats: bool,
}

impl Simulation {
    pub fn new() -> Simulation {
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_recv) = crossbeam::channel::unbounded();
        Simulation {
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
            event_collector: ChannelEventCollector::new(intersection_send, contact_send),
            contact_recv,
            intersection_recv,
            events: SimEvents::new(),
            tick: 0,
            cheats: false,
        }
    }

    pub fn time(&self) -> f64 {
        self.tick as f64 * PHYSICS_TICK_LENGTH
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

        while self.intersection_recv.try_recv().is_ok() {}

        let handle_snapshot: Vec<ShipHandle> = self.ships.iter().cloned().collect();
        for handle in handle_snapshot {
            if let Some(ship_controller) = self.ship_controllers.get_mut(&handle) {
                if let Err(e) = ship_controller.tick() {
                    self.events.errors.push(e);
                }
            }
            self.ship_mut(handle).tick();
        }

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
}

impl Default for Simulation {
    fn default() -> Self {
        Simulation::new()
    }
}

pub struct CollisionEventHandler {
    pub collision: crossbeam::atomic::AtomicCell<bool>,
}

impl CollisionEventHandler {
    pub fn new() -> CollisionEventHandler {
        CollisionEventHandler {
            collision: crossbeam::atomic::AtomicCell::new(false),
        }
    }
}

impl EventHandler for CollisionEventHandler {
    fn handle_intersection_event(&self, _event: IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent, _contact_pair: &ContactPair) {
        if let ContactEvent::Started(_, _) = event {
            self.collision.store(true);
            //println!("Collision: {:?}", event);
        }
    }
}

impl Default for CollisionEventHandler {
    fn default() -> Self {
        CollisionEventHandler::new()
    }
}

#[derive(Clone)]
pub struct Line {
    pub a: Point2<f64>,
    pub b: Point2<f64>,
    pub color: Vector4<f32>,
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
    }
}

impl Default for SimEvents {
    fn default() -> Self {
        SimEvents::new()
    }
}
