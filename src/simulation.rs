use rapier2d_f64::prelude::*;

pub struct Ball {
    pub body: RigidBodyHandle,
    pub r: f32,
}

pub const WORLD_SIZE: f32 = 1000.0;

pub struct Simulation {
    pub balls: Vec<Ball>,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
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
            balls: vec![],
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

        let mut make_edge = |x: f32, y: f32, a: f32| {
            let edge_length = WORLD_SIZE;
            let edge_width = 1.0;
            let rigid_body = RigidBodyBuilder::new_static()
                .translation(vector![x.into(), y.into()])
                .rotation(a.into())
                .build();
            let handle = sim.bodies.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(edge_length as f64, edge_width)
                .restitution(1.0)
                .build();
            sim.colliders
                .insert_with_parent(collider, handle, &mut sim.bodies);
        };
        make_edge(0.0, WORLD_SIZE / 2.0, 0.0);
        make_edge(0.0, -WORLD_SIZE / 2.0, 0.0);
        make_edge(WORLD_SIZE / 2.0, 0.0, std::f32::consts::PI / 2.0);
        make_edge(-WORLD_SIZE / 2.0, 0.0, std::f32::consts::PI / 2.0);

        sim
    }

    pub fn add_ball(self: &mut Simulation, x: f32, y: f32, vx: f32, vy: f32, r: f32) {
        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(vector![x.into(), y.into()])
            .linvel(vector![vx.into(), vy.into()])
            .build();
        let handle = self.bodies.insert(rigid_body);
        let collider = ColliderBuilder::ball(r.into())
            .restitution(1.0)
            .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
            .build();
        self.colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        self.balls.push(Ball { body: handle, r });
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
