use nalgebra::vector;
use oort_simulator::ship::{fighter, frigate, target};
use oort_simulator::simulation::{self, Code};
use oort_simulator::{bullet, ship};
use test_log::test;

#[test]
fn test_hit() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);

    let ship0 = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.1,
        fighter(1),
    );

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    let initial_health = sim.ship(ship1).data().health;

    sim.ship_mut(ship0).fire_gun(0);
    assert!(!sim.bullets.iter().len() > 0);

    for _ in 0..100 {
        sim.step();
    }

    assert!(sim.bullets.iter().len() == 0);
    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    assert_ne!(sim.ship(ship1).data().health, initial_health);
}

#[test]
fn test_destroyed() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);

    let ship0 = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.1,
        target(1),
    );

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    for _ in 0..1000 {
        sim.ship_mut(ship0).fire_gun(0);
        sim.step();
        if !sim.ships.contains(ship1) {
            break;
        }
    }

    assert!(sim.ships.contains(ship0));
    assert!(!sim.ships.contains(ship1));
}

#[test]
fn test_penetration() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);

    let ship0 = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        frigate(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.1,
        target(1),
    );

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    sim.ship_mut(ship0).fire_gun(0);
    let bullet = *sim.bullets.iter().next().unwrap();
    let initial_bullet_mass = bullet::data(&sim, bullet).mass;
    let initial_velocity = *bullet::body(&sim, bullet).linvel();

    for _ in 0..100 {
        sim.step();
    }

    assert!(sim.bullets.iter().len() == 1);
    assert!(sim.ships.contains(ship0));
    assert!(!sim.ships.contains(ship1));
    assert_ne!(bullet::data(&sim, bullet).mass, initial_bullet_mass);
    assert_ne!(*bullet::body(&sim, bullet).linvel(), initial_velocity);
}
