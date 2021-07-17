use oort::simulation;
use oort::simulation::ship;
use oort::simulation::ship::fighter;
use test_env_log::test;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new();

    let ship0 = ship::create(&mut sim, -100.0, 0.0, 0.0, 0.0, 0.0, fighter());
    let ship1 = ship::create(&mut sim, 100.0, 0.0, 0.0, 0.0, 0.1, fighter());

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    sim.ship_mut(ship0).fire_weapon(0);
    assert!(!sim.bullets.iter().len() > 0);

    for _ in 0..100 {
        sim.step();
    }

    assert!(sim.bullets.iter().len() == 0);
    assert!(sim.ships.contains(ship0));
    assert!(!sim.ships.contains(ship1));
}
