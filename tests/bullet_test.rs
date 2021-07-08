use oort::simulation;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    let ship0 = oort::ship::create(&mut sim, -100.0, 0.0, 0.0, 0.0, 0.0);
    let ship1 = oort::ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.1);

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    sim.ship_mut(ship0).fire_weapon();
    assert!(!sim.bullets.iter().len() > 0);

    for _ in 0..100 {
        sim.step();
    }

    assert!(sim.bullets.iter().len() == 0);
    assert!(sim.ships.contains(ship0));
    assert!(!sim.ships.contains(ship1));
}
