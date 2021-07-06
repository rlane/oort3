use oort::simulation;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    let ship0 = sim.add_ship(-100.0, 0.0, 0.0, 0.0, 0.0);
    let ship1 = sim.add_ship(100.0, 0.0, 0.0, 0.0, 0.1);

    assert_eq!(sim.ships[&ship0].velocity(&sim).magnitude(), 0.0);
    assert_eq!(sim.ships[&ship1].velocity(&sim).magnitude(), 0.0);

    sim.fire_weapon(sim.ships[&ship0].body);
    for _ in 0..1000 {
        sim.step();
    }

    assert_eq!(sim.ships[&ship0].velocity(&sim).magnitude(), 0.0);
    assert_ne!(sim.ships[&ship1].velocity(&sim).magnitude(), 0.0);
}
