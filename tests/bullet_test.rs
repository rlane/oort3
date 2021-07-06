use oort::simulation;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    sim.add_ship(-100.0, 0.0, 0.0, 0.0, 0.0);
    sim.add_ship(100.0, 0.0, 0.0, 0.0, 0.1);

    assert_eq!(sim.ships[0].velocity(&sim).length(), 0.0);
    assert_eq!(sim.ships[1].velocity(&sim).length(), 0.0);

    sim.fire_weapon(sim.ships[0].body);
    for _ in 0..1000 {
        sim.step();
    }

    assert_eq!(sim.ships[0].velocity(&sim).length(), 0.0);
    assert_ne!(sim.ships[1].velocity(&sim).length(), 0.0);
}
