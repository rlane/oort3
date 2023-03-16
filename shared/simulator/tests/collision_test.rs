use nalgebra::vector;
use oort_simulator::ship::{fighter, missile};
use oort_simulator::simulation::{self, Code};
use oort_simulator::{bullet, collision, ship};
use rand::Rng;
use test_log::test;

const BULLET_COLOR: u32 = 0xffffffff;

#[test]
fn test_world_edge() {
    let mut rng = rand::thread_rng();
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);
    collision::add_walls(&mut sim);

    for _ in 0..100 {
        let s = 500.0;
        let r = rng.gen_range(10.0..20.0);
        let x = rng.gen_range((r - sim.world_size() / 2.0)..(sim.world_size() / 2.0 - r));
        let y = rng.gen_range((r - sim.world_size() / 2.0)..(sim.world_size() / 2.0 - r));
        let h = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
        let vx = rng.gen_range(-s..s);
        let vy = rng.gen_range(-s..s);
        ship::create(
            &mut sim,
            vector![x, y],
            vector![vx, vy],
            h as f64,
            fighter(0),
        );
    }

    for _ in 0..1000 {
        sim.step();
    }

    for &index in sim.ships.iter() {
        let ship = sim.ship(index);
        assert!(ship.position().x >= -sim.world_size() / 2.0);
        assert!(ship.position().x <= sim.world_size() / 2.0);
        assert!(ship.position().y >= -sim.world_size() / 2.0);
        assert!(ship.position().y <= sim.world_size() / 2.0);
    }
}

#[test]
fn test_head_on_collision() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    let ship0 = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![100.0, 0.0],
        0.0,
        fighter(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![-100.0, 0.0],
        0.0,
        fighter(0),
    );

    assert!(sim.ship(ship0).velocity().x > 0.0);
    assert!(sim.ship(ship1).velocity().x < 0.0);

    for _ in 0..1000 {
        sim.step();
    }

    assert!(sim.ship(ship0).velocity().x < 0.0);
    assert!(sim.ship(ship1).velocity().x > 0.0);
}

#[test]
fn test_fighter_bullet_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    let ship = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    bullet::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![1000.0, 0.0],
        bullet::BulletData {
            mass: 0.1,
            team: 0,
            color: BULLET_COLOR,
            ttl: 5.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ship(ship).velocity().x, 0.0);
    assert_eq!(sim.bullets.len(), 1);
}

#[test]
fn test_fighter_bullet_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    let ship = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    bullet::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![1000.0, 0.0],
        bullet::BulletData {
            mass: 0.1,
            team: 1,
            color: BULLET_COLOR,
            ttl: 5.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ship(ship).velocity().x, 0.0);
    assert_eq!(sim.bullets.len(), 0);
}

#[test]
fn test_missile_bullet_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    let msl = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        missile(0),
    );
    let blt = bullet::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![1000.0, 0.0],
        bullet::BulletData {
            mass: 0.1,
            team: 0,
            color: BULLET_COLOR,
            ttl: 5.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ship(msl).velocity().x, 0.0);
    assert_eq!(bullet::body(&mut sim, blt).linvel().x, 1000.0);
}

#[test]
fn test_missile_bullet_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        missile(0),
    );
    bullet::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![1000.0, 0.0],
        bullet::BulletData {
            mass: 10.0,
            team: 1,
            color: BULLET_COLOR,
            ttl: 5.0,
        },
    );

    for _ in 0..60 {
        sim.step();
    }

    assert_eq!(sim.ships.len(), 0);
    assert_eq!(sim.bullets.len(), 1);
}

#[test]
fn test_missile_fighter_collision_same_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    let msl = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![400.0, 0.0],
        0.0,
        missile(0),
    );
    let ship = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );

    for _ in 0..60 {
        sim.step();
    }

    assert!(sim.ship(ship).exists());
    assert!(sim.ship(msl).exists());
}

#[test]
fn test_missile_fighter_collision_different_team() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);

    let msl = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![400.0, 0.0],
        0.0,
        missile(0),
    );
    let ship = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(1),
    );

    for _ in 0..60 {
        sim.step();
    }

    assert!(!sim.ship(ship).exists());
    assert!(!sim.ship(msl).exists());
}

#[test]
fn test_bullet_continuous_collision_detection() {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None]);

    for i in 0..16 {
        let offset = i as f64;
        ship::create(
            &mut sim,
            vector![-1000.0 + offset, 0.0],
            vector![1000.0, 0.0],
            0.0,
            missile(0),
        );
        bullet::create(
            &mut sim,
            vector![0.0, -1000.0 + offset],
            vector![0.0, 1000.0],
            bullet::BulletData {
                mass: 10.0,
                team: 1,
                color: BULLET_COLOR,
                ttl: 1.5,
            },
        );

        for _ in 0..120 {
            sim.step();
        }

        assert_eq!(sim.ships.len(), 0);
        assert_eq!(sim.bullets.len(), 0);
    }
}
