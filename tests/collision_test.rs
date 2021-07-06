use macroquad::rand;
use oort::simulation;
use oort::simulation::WORLD_SIZE;

#[test]
fn test_world_edge() {
    let mut sim = oort::simulation::Simulation::new();

    for _ in 0..100 {
        let s = 500.0;
        let r = rand::gen_range(10.0, 20.0);
        let x = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let y = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let h = rand::gen_range(0.0, 2.0 * std::f32::consts::PI);
        let vx = rand::gen_range(-s, s);
        let vy = rand::gen_range(-s, s);
        sim.add_ship(x, y, vx, vy, h);
    }

    for _ in 0..1000 {
        sim.step();
    }

    for ship in &sim.ships {
        assert!(ship.position(&sim).x >= -WORLD_SIZE / 2.0);
        assert!(ship.position(&sim).x <= WORLD_SIZE / 2.0);
        assert!(ship.position(&sim).y >= -WORLD_SIZE / 2.0);
        assert!(ship.position(&sim).y <= WORLD_SIZE / 2.0);
    }
}

#[test]
fn test_head_on_collision() {
    let mut sim = simulation::Simulation::new();

    sim.add_ship(-100.0, 0.0, 100.0, 0.0, 0.0);
    sim.add_ship(100.0, 0.0, -100.0, 0.0, 0.0);

    assert!(sim.ships[0].velocity(&sim).x > 0.0);
    assert!(sim.ships[1].velocity(&sim).x < 0.0);

    for _ in 0..1000 {
        sim.step();
    }

    assert!(sim.ships[0].velocity(&sim).x < 0.0);
    assert!(sim.ships[1].velocity(&sim).x > 0.0);
}
