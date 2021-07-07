use crate::bullet::{BulletAccessor, BulletHandle};
use crate::index_set::IndexSet;
use crate::ship::{ShipAccessor, ShipAccessorMut, ShipHandle};
use rapier2d_f64::prelude::*;

pub const WORLD_SIZE: f64 = 1000.0;

pub(crate) const WALL_COLLISION_GROUP: u32 = 0;
pub(crate) const SHIP_COLLISION_GROUP: u32 = 1;
pub(crate) const BULLET_COLLISION_GROUP: u32 = 2;

pub struct Simulation {
    pub ships: IndexSet<ShipHandle>,
    pub bullets: IndexSet<BulletHandle>,
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    joints: JointSet,
    pub collision_event_handler: CollisionEventHandler,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
}

impl Simulation {
    pub fn new() -> Simulation {
        let mut sim = Simulation {
            ships: IndexSet::new(),
            bullets: IndexSet::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            collision_event_handler: CollisionEventHandler::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
        };

        let mut make_edge = |x: f64, y: f64, a: f64| {
            let edge_length = WORLD_SIZE as f64;
            let edge_width = 10.0;
            let rigid_body = RigidBodyBuilder::new_static()
                .translation(vector![x, y])
                .rotation(a)
                .build();
            let body_handle = sim.bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(edge_length / 2.0, edge_width / 2.0)
                .restitution(1.0)
                .collision_groups(InteractionGroups::new(
                    1 << WALL_COLLISION_GROUP,
                    1 << SHIP_COLLISION_GROUP | 1 << BULLET_COLLISION_GROUP,
                ))
                .build();
            sim.colliders
                .insert_with_parent(collider, body_handle, &mut sim.bodies);
        };
        make_edge(0.0, WORLD_SIZE / 2.0, 0.0);
        make_edge(0.0, -WORLD_SIZE / 2.0, std::f64::consts::PI);
        make_edge(WORLD_SIZE / 2.0, 0.0, std::f64::consts::PI / 2.0);
        make_edge(-WORLD_SIZE / 2.0, 0.0, 3.0 * std::f64::consts::PI / 2.0);

        sim
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
            &self.collision_event_handler,
        );
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
