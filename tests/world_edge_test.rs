use macroquad::rand;
use oort::simulation::WORLD_SIZE;

#[test]
fn test_world_edge() {
    let mut sim = oort::simulation::Simulation::new();

    for _ in 0..100 {
        let s = 500.0;
        let r = rand::gen_range(10.0, 20.0);
        let x = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let y = rand::gen_range(r - WORLD_SIZE / 2.0, WORLD_SIZE / 2.0 - r);
        let vx = rand::gen_range(-s, s);
        let vy = rand::gen_range(-s, s);
        sim.add_ship(x, y, vx, vy);
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
