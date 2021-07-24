pub mod bullet;
pub mod index_set;
pub mod scenario;
pub mod ship;

use self::bullet::{BulletAccessor, BulletAccessorMut, BulletData, BulletHandle};
use self::index_set::IndexSet;
use self::ship::{ShipAccessor, ShipAccessorMut, ShipData, ShipHandle};
use crate::script;
use crate::script::ShipController;
use rapier2d_f64::prelude::*;
use std::collections::HashMap;

pub const WORLD_SIZE: f64 = 10000.0;
pub(crate) const PHYSICS_TICK_LENGTH: f64 = 1.0 / 60.0;

pub(crate) const WALL_COLLISION_GROUP: u32 = 0;
pub(crate) const SHIP_COLLISION_GROUP: u32 = 1;
pub(crate) const BULLET_COLLISION_GROUP: u32 = 2;

pub struct Simulation {
    pub ships: IndexSet<ShipHandle>,
    pub(crate) ship_data: HashMap<ShipHandle, ShipData>,
    pub(crate) ship_controllers: HashMap<ShipHandle, Box<dyn ShipController>>,
    pub bullets: IndexSet<BulletHandle>,
    pub(crate) bullet_data: HashMap<BulletHandle, BulletData>,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) joints: JointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    event_collector: rapier2d_f64::pipeline::ChannelEventCollector,
    contact_recv: crossbeam::channel::Receiver<ContactEvent>,
    intersection_recv: crossbeam::channel::Receiver<IntersectionEvent>,
    pub collided: bool,
    pub errors: Vec<script::Error>,
}

impl Simulation {
    pub fn new() -> Simulation {
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_recv) = crossbeam::channel::unbounded();
        Simulation {
            ships: IndexSet::new(),
            ship_data: HashMap::new(),
            ship_controllers: HashMap::new(),
            bullets: IndexSet::new(),
            bullet_data: HashMap::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            integration_parameters: IntegrationParameters {
                dt: PHYSICS_TICK_LENGTH,
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
            collided: false,
            errors: Vec::new(),
        }
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
                    let damage = sim.bullet(bullet).data().damage;
                    let ship_destroyed = {
                        let ship_data = sim.ship_data.get_mut(&ship).unwrap();
                        ship_data.health -= damage;
                        ship_data.health <= 0.0
                    };
                    if ship_destroyed {
                        sim.ship_mut(ship).explode();
                    }
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

                self.collided = true;
            }
        }

        while self.intersection_recv.try_recv().is_ok() {}

        self.errors.clear();
        let handle_snapshot: Vec<ShipHandle> = self.ships.iter().cloned().collect();
        for handle in handle_snapshot {
            self.ship_mut(handle).tick();
            if let Some(ship_controller) = self.ship_controllers.get_mut(&handle) {
                if let Err(e) = ship_controller.tick() {
                    self.errors.push(e);
                }
            }
        }
    }

    pub fn upload_code(&mut self, code: &str, team: i32) {
        for &handle in self.ships.iter() {
            if self.ship(handle).data().team == team {
                if let Some(ship_controller) = self.ship_controllers.get_mut(&handle) {
                    ship_controller.upload_code(code);
                    ship_controller.start();
                }
            }
        }
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
