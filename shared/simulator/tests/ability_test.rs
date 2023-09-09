use nalgebra::vector;
use oort_api::Ability;
use oort_simulator::ship;
use oort_simulator::ship::{cruiser, fighter, frigate, torpedo, ShipClass};
use oort_simulator::simulation::{self, Code, PHYSICS_TICK_LENGTH};
use test_log::test;

#[test]
fn test_boost() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);
    let mut prev_v = vector![0.0, 0.0];
    let ship0 = ship::create(&mut sim, vector![0.0, 0.0], prev_v, 0.0, fighter(0));

    sim.step();
    let v = sim.ship(ship0).velocity();
    let acc = (v - prev_v) / PHYSICS_TICK_LENGTH;
    prev_v = v;
    approx::assert_abs_diff_eq!(acc.magnitude(), 0.0, epsilon = 1.0);

    sim.ship_mut(ship0).activate_ability(Ability::Boost);
    sim.step();

    for _ in 0..120 {
        sim.step();
        let v = sim.ship(ship0).velocity();
        let acc = (v - prev_v) / PHYSICS_TICK_LENGTH;
        prev_v = v;
        approx::assert_abs_diff_eq!(acc.magnitude(), 100.0, epsilon = 1.0);
    }

    sim.step();
    let v = sim.ship(ship0).velocity();
    let acc = (v - prev_v) / PHYSICS_TICK_LENGTH;
    approx::assert_abs_diff_eq!(acc.magnitude(), 0.0, epsilon = 1.0);
}

#[test]
fn test_deactivate_boost() {
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
    assert!(sim.ship(ship0).is_ability_active(Ability::Boost));
    sim.ship_mut(ship0).accelerate(vector![50.0, 0.0]);
    sim.ship_mut(ship0).tick();
    sim.step();
    let v2 = sim.ship(ship0).velocity();
    let acc = (v2 - v1) / PHYSICS_TICK_LENGTH;
    approx::assert_abs_diff_eq!(acc.magnitude(), 150.0, epsilon = 1.0);

    sim.ship_mut(ship0).deactivate_ability(Ability::Boost);
    assert!(!sim.ship(ship0).is_ability_active(Ability::Boost));
    sim.ship_mut(ship0).accelerate(vector![50.0, 0.0]);
    sim.ship_mut(ship0).tick();
    sim.step();
    let v3 = sim.ship(ship0).velocity();
    let acc = (v3 - v2) / PHYSICS_TICK_LENGTH;
    approx::assert_abs_diff_eq!(acc.magnitude(), 50.0, epsilon = 1.0);
}

#[test]
fn test_decoy() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);
    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        torpedo(1),
    );

    sim.ship_mut(ship1).activate_ability(Ability::Decoy);
    sim.step();

    assert_eq!(
        sim.ship(ship0)
            .radar()
            .as_ref()
            .unwrap()
            .scan()
            .unwrap()
            .class,
        ShipClass::Cruiser
    );
}

#[test]
fn test_shield() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);
    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        frigate(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![1000.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        cruiser(1),
    );

    sim.ship_mut(ship1).activate_ability(Ability::Shield);
    sim.ship_mut(ship0).fire(0);

    for _ in 0..30 {
        sim.step();
    }

    assert_ne!(sim.ship(ship0).data().health, frigate(0).health);
    assert_eq!(sim.ship(ship1).data().health, cruiser(1).health);
}
