use macroquad::rand;
use oort::simulation;
use oort::simulation::WORLD_SIZE;
use rapier2d_f64::dynamics::RigidBodyHandle;

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
        sim.add_ship(x as f64, y as f64, vx as f64, vy as f64, h as f64);
    }

    for _ in 0..1000 {
        sim.step();
    }

    for &index in sim.ships.iter() {
        let body = sim.bodies.get(RigidBodyHandle(index)).unwrap();
        assert!(body.position().translation.x >= -WORLD_SIZE / 2.0);
        assert!(body.position().translation.x <= WORLD_SIZE / 2.0);
        assert!(body.position().translation.y >= -WORLD_SIZE / 2.0);
        assert!(body.position().translation.y <= WORLD_SIZE / 2.0);
    }
}

#[test]
fn test_head_on_collision() {
    let mut sim = simulation::Simulation::new();

    let ship0 = sim.add_ship(-100.0, 0.0, 100.0, 0.0, 0.0);
    let ship1 = sim.add_ship(100.0, 0.0, -100.0, 0.0, 0.0);

    assert!(sim.bodies.get(ship0).unwrap().linvel().x > 0.0);
    assert!(sim.bodies.get(ship1).unwrap().linvel().x < 0.0);

    for _ in 0..1000 {
        sim.step();
    }

    assert!(sim.bodies.get(ship0).unwrap().linvel().x < 0.0);
    assert!(sim.bodies.get(ship1).unwrap().linvel().x > 0.0);
}
