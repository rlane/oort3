use macroquad::rand;
use oort::ship::ShipClass;
use oort::simulation;
use oort::simulation::WORLD_SIZE;

#[test]
fn test_world_edge() {
    let mut sim = oort::simulation::Simulation::new();
    oort::scenario::add_walls(&mut sim);

    for _ in 0..100 {
        let s = 500.0;
        let r = rand::gen_range(10.0, 20.0);
        let x = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let y = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let h = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
        let vx = rand::gen_range(-s, s);
        let vy = rand::gen_range(-s, s);
        oort::ship::create(
            &mut sim,
            x as f64,
            y as f64,
            vx as f64,
            vy as f64,
            h as f64,
            ShipClass::Fighter,
        );
    }

    for _ in 0..1000 {
        sim.step();
    }

    for &index in sim.ships.iter() {
        let ship = sim.ship(index);
        assert!(ship.position().x >= -WORLD_SIZE / 2.0);
        assert!(ship.position().x <= WORLD_SIZE / 2.0);
        assert!(ship.position().y >= -WORLD_SIZE / 2.0);
        assert!(ship.position().y <= WORLD_SIZE / 2.0);
    }
}

#[test]
fn test_head_on_collision() {
    let mut sim = simulation::Simulation::new();

    let ship0 = oort::ship::create(&mut sim, -100.0, 0.0, 100.0, 0.0, 0.0, ShipClass::Fighter);
    let ship1 = oort::ship::create(&mut sim, 100.0, 0.0, -100.0, 0.0, 0.0, ShipClass::Fighter);

    assert!(sim.ship(ship0).velocity().x > 0.0);
    assert!(sim.ship(ship1).velocity().x < 0.0);

    for _ in 0..1000 {
        sim.step();
    }

    assert!(sim.ship(ship0).velocity().x < 0.0);
    assert!(sim.ship(ship1).velocity().x > 0.0);
}
