use nalgebra::vector;
use oort_simulator::ship;
use oort_simulator::ship::{cruiser, fighter, frigate, target};
use oort_simulator::simulation::{self, Code};
use test_log::test;

fn high_health_target(team: i32) -> ship::ShipData {
    let mut data = target(team);
    data.health = 1e6;
    data
}

fn find_gun_dps(mut ship_data: ship::ShipData, gun: i64) -> f64 {
    let mut sim = simulation::Simulation::new("test", 0, &[Code::None, Code::None]);

    let offset = ship_data.guns[gun as usize].offset;
    ship_data.guns[gun as usize].inaccuracy = 0.0;

    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        ship_data,
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0] + offset,
        vector![0.0, 0.0],
        0.0,
        high_health_target(1),
    );

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));

    let initial_health = sim.ship(ship1).data().health;

    let ticks = 600;
    let time = ticks as f64 * simulation::PHYSICS_TICK_LENGTH;

    for _ in 0..ticks {
        sim.ship_mut(ship0).fire_gun(gun);
        sim.step();
    }

    for _ in 0..100 {
        sim.step();
    }

    assert!(sim.ships.contains(ship0));
    assert!(sim.ships.contains(ship1));
    assert!(sim.bullets.is_empty());

    (initial_health - sim.ship(ship1).data().health) / time
}

#[test]
fn test_dps() {
    approx::assert_abs_diff_eq!(find_gun_dps(fighter(0), 0), 105.0, epsilon = 1.0);
    approx::assert_abs_diff_eq!(find_gun_dps(frigate(0), 0), 560.0, epsilon = 1.0);
    approx::assert_abs_diff_eq!(find_gun_dps(frigate(0), 1), 105.0, epsilon = 1.0);
    approx::assert_abs_diff_eq!(find_gun_dps(frigate(0), 2), 105.0, epsilon = 1.0);
    approx::assert_abs_diff_eq!(find_gun_dps(cruiser(0), 0), 106.0, epsilon = 1.0);
}
