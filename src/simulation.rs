use crate::index_set::{HasIndex, Index, IndexSet};
use crate::ship::{ShipAccessor, ShipHandle};
use rapier2d_f64::prelude::*;

pub const WORLD_SIZE: f64 = 1000.0;

const WALL_COLLISION_GROUP: u32 = 0;
const SHIP_COLLISION_GROUP: u32 = 1;
const BULLET_COLLISION_GROUP: u32 = 2;

pub struct Simulation {
    pub ships: IndexSet<ShipHandle>,
    pub bullets: IndexSet<BulletHandle>,
    pub(crate) bodies: RigidBodySet,
    colliders: ColliderSet,
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

    pub fn add_ship(self: &mut Simulation, x: f64, y: f64, vx: f64, vy: f64, h: f64) -> ShipHandle {
        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![x, y])
            .linvel(vector![vx, vy])
            .rotation(h)
            .ccd_enabled(true)
            .build();
        let body_handle = self.bodies.insert(rigid_body);
        let vertices = crate::model::ship()
            .iter()
            .map(|&v| point![v.x as f64, v.y as f64])
            .collect::<Vec<_>>();
        let collider = ColliderBuilder::convex_hull(&vertices)
            .unwrap()
            .restitution(1.0)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .collision_groups(InteractionGroups::new(
                1 << SHIP_COLLISION_GROUP,
                1 << WALL_COLLISION_GROUP | 1 << SHIP_COLLISION_GROUP | 1 << BULLET_COLLISION_GROUP,
            ))
            .build();
        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies);
        let handle = ShipHandle(body_handle.0);
        self.ships.insert(handle);
        handle
    }

    pub fn ship(self: &Simulation, handle: ShipHandle) -> ShipAccessor {
        ShipAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn fire_weapon(self: &mut Simulation, handle: ShipHandle) {
        let body = self.bodies.get(RigidBodyHandle(handle.index())).unwrap();
        let x = body.position().translation.x;
        let y = body.position().translation.y;
        let v2 = body.position().rotation.into_inner() * 1000.0;
        let vx = body.linvel().x + v2.re;
        let vy = body.linvel().y + v2.im;
        self.add_bullet(x as f64, y as f64, vx as f64, vy as f64);
    }

    pub fn add_bullet(self: &mut Simulation, x: f64, y: f64, vx: f64, vy: f64) {
        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![x, y])
            .linvel(vector![vx, vy])
            .ccd_enabled(true)
            .build();
        let body_handle = self.bodies.insert(rigid_body);
        let collider = ColliderBuilder::ball(1.0)
            .restitution(1.0)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .collision_groups(InteractionGroups::new(
                1 << BULLET_COLLISION_GROUP,
                1 << WALL_COLLISION_GROUP | 1 << SHIP_COLLISION_GROUP,
            ))
            .build();
        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.bullets.insert(BulletHandle(body_handle.0));
    }

    pub fn bullet(self: &Simulation, handle: BulletHandle) -> BulletAccessor {
        BulletAccessor {
            simulation: self,
            handle,
        }
    }

    pub fn thrust_main(self: &mut Simulation, handle: ShipHandle, force: f64) {
        let body = self
            .bodies
            .get_mut(RigidBodyHandle(handle.index()))
            .unwrap();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![force, 0.0], true);
    }

    pub fn thrust_lateral(self: &mut Simulation, handle: ShipHandle, force: f64) {
        let body = self
            .bodies
            .get_mut(RigidBodyHandle(handle.index()))
            .unwrap();
        let rotation_matrix = body.position().rotation.to_rotation_matrix();
        body.apply_force(rotation_matrix * vector![0.0, force], true);
    }

    pub fn thrust_angular(self: &mut Simulation, handle: ShipHandle, torque: f64) {
        let body = self
            .bodies
            .get_mut(RigidBodyHandle(handle.index()))
            .unwrap();
        body.apply_torque(torque, true);
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

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct BulletHandle(pub Index);

impl HasIndex for BulletHandle {
    fn index(self) -> Index {
        self.0
    }
}

pub struct BulletAccessor<'a> {
    simulation: &'a Simulation,
    handle: BulletHandle,
}

impl<'a> BulletAccessor<'a> {
    pub fn body(&self) -> &'a RigidBody {
        self.simulation
            .bodies
            .get(RigidBodyHandle(self.handle.index()))
            .unwrap()
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
