use oort::simulation;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    let ship0 = sim.add_ship(-100.0, 0.0, 0.0, 0.0, 0.0);
    let ship1 = sim.add_ship(100.0, 0.0, 0.0, 0.0, 0.1);

    assert_eq!(sim.ship(ship0).velocity().magnitude(), 0.0);
    assert_eq!(sim.ship(ship1).velocity().magnitude(), 0.0);

    sim.ship_mut(ship0).fire_weapon();
    for _ in 0..1000 {
        sim.step();
    }

    assert_eq!(sim.ship(ship0).velocity().magnitude(), 0.0);
    assert_ne!(sim.ship(ship1).velocity().magnitude(), 0.0);
}
