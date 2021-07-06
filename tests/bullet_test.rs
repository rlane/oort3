use oort::simulation;
use rapier2d_f64::dynamics::RigidBodyHandle;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    let ship0 = sim.add_ship(-100.0, 0.0, 0.0, 0.0, 0.0);
    let ship1 = sim.add_ship(100.0, 0.0, 0.0, 0.0, 0.1);

    assert_eq!(
        sim.bodies
            .get(RigidBodyHandle(ship0))
            .unwrap()
            .linvel()
            .magnitude(),
        0.0
    );
    assert_eq!(
        sim.bodies
            .get(RigidBodyHandle(ship1))
            .unwrap()
            .linvel()
            .magnitude(),
        0.0
    );

    sim.fire_weapon(ship0);
    for _ in 0..1000 {
        sim.step();
    }

    assert_eq!(
        sim.bodies
            .get(RigidBodyHandle(ship0))
            .unwrap()
            .linvel()
            .magnitude(),
        0.0
    );
    assert_ne!(
        sim.bodies
            .get(RigidBodyHandle(ship1))
            .unwrap()
            .linvel()
            .magnitude(),
        0.0
    );
}
