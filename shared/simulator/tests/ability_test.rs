use nalgebra::vector;
use oort_api::Ability;
use oort_simulator::ship;
use oort_simulator::ship::{fighter, missile};
use oort_simulator::simulation::{self, Code, PHYSICS_TICK_LENGTH};
use test_log::test;

#[test]
fn test_boost() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);
    let v0 = vector![0.0, 0.0];
    let ship0 = ship::create(&mut sim, vector![0.0, 0.0], v0, 0.0, fighter(0));

    sim.ship_mut(ship0).accelerate(vector![50.0, 0.0]);
    sim.ship_mut(ship0).tick();
    sim.step();
    let v1 = sim.ship(ship0).velocity();
    let acc = (v1 - v0) / PHYSICS_TICK_LENGTH;
    approx::assert_abs_diff_eq!(acc.magnitude(), 50.0, epsilon = 1.0);

    sim.ship_mut(ship0).activate_ability(Ability::Boost);
    sim.ship_mut(ship0).accelerate(vector![50.0, 0.0]);
    sim.ship_mut(ship0).tick();
    sim.step();
    let v2 = sim.ship(ship0).velocity();
    let acc = (v2 - v1) / PHYSICS_TICK_LENGTH;
    approx::assert_abs_diff_eq!(acc.magnitude(), 150.0, epsilon = 1.0);
}

#[test]
fn test_shaped_charge() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);
    let v0 = vector![0.0, 0.0];
    let ship0 = ship::create(&mut sim, vector![0.0, 0.0], v0, 0.0, missile(0));

    sim.ship_mut(ship0).activate_ability(Ability::ShapedCharge);
    sim.ship_mut(ship0).explode();
    sim.step();

    assert!(!sim.bullets.is_empty());
    for &handle in sim.bullets.iter() {
        let bullet = sim.bullet(handle);
        let v = *bullet.body().linvel();
        let max_angle = 0.05;
        assert!(v.angle(&vector![1.0, 0.0]) <= max_angle);
        assert!(v.angle(&vector![1.0, 0.0]) >= -max_angle);
    }
}
