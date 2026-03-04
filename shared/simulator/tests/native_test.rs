/// Integration tests for the native (non-WASM) team controller.
///
/// These adapt existing WASM-based tests to exercise Code::Native, verifying
/// that per-ship global state save/restore works correctly when multiple
/// ships share the same process memory.
use nalgebra::vector;
use oort_simulator::ship::{self, fighter, ShipHandle};
use oort_simulator::simulation::{self, Code, NativeShip};
use serial_test::serial;
use std::sync::Arc;
use test_log::test;

// ---------------------------------------------------------------------------
// Helper: extract debug text for a ship handle from SimEvents
// ---------------------------------------------------------------------------
fn debug_text(sim: &simulation::Simulation, handle: ShipHandle) -> String {
    sim.events()
        .debug_text
        .get(&handle.into())
        .cloned()
        .unwrap_or_default()
}

// ===========================================================================
// 1. Multi-ship ID assignment (adapts api_test::test_id)
//
// Creates 3 ships across 2 teams, each reporting its id() via debug!().
// Verifies that per-team IDs are assigned sequentially and independently.
// ===========================================================================

struct IdShip {}

impl NativeShip for IdShip {
    fn tick(&mut self) {
        oort_api::prelude::debug!("ID:{}", oort_api::prelude::id());
    }
}

#[test]
#[serial]
fn test_native_multi_ship_ids() {
    let factory: simulation::NativeShipFactory = Arc::new(|| Box::new(IdShip {}));
    let code = Code::Native(factory);

    let mut sim = simulation::Simulation::new("test", 0, &[code.clone(), code]);

    // 2 ships on team 0, 1 ship on team 1
    let h0 = ship::create(&mut sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));
    let h1 = ship::create(&mut sim, vector![100.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));
    let h2 = ship::create(&mut sim, vector![200.0, 0.0], vector![0.0, 0.0], 0.0, fighter(1));

    sim.step();

    let text0 = debug_text(&sim, h0);
    let text1 = debug_text(&sim, h1);
    let text2 = debug_text(&sim, h2);

    assert!(text0.contains("ID:1"), "ship0 should have ID 1, got: {text0}");
    assert!(text1.contains("ID:2"), "ship1 should have ID 2, got: {text1}");
    // Team 1's first ship gets ID 1 independently
    assert!(text2.contains("ID:1"), "ship2 (team 1) should have ID 1, got: {text2}");
}

// ===========================================================================
// 2. Per-ship state isolation (position, velocity, heading)
//
// Two ships at different positions each report their own position via debug.
// If state save/restore is broken, ship B would see ship A's position.
// ===========================================================================

struct PositionReportShip {}

impl NativeShip for PositionReportShip {
    fn tick(&mut self) {
        let p = oort_api::prelude::position();
        let v = oort_api::prelude::velocity();
        let h = oort_api::prelude::heading();
        oort_api::prelude::debug!(
            "POS:{:.0},{:.0} VEL:{:.0},{:.0} HDG:{:.2}",
            p.x, p.y, v.x, v.y, h
        );
    }
}

#[test]
#[serial]
fn test_native_state_isolation() {
    let factory: simulation::NativeShipFactory = Arc::new(|| Box::new(PositionReportShip {}));
    let code = Code::Native(factory);

    let mut sim = simulation::Simulation::new("test", 0, &[code, Code::None]);

    let h0 = ship::create(
        &mut sim,
        vector![1000.0, 2000.0],
        vector![10.0, 20.0],
        1.0,
        fighter(0),
    );
    let h1 = ship::create(
        &mut sim,
        vector![-500.0, -800.0],
        vector![-5.0, -8.0],
        -2.0,
        fighter(0),
    );

    sim.step();

    let text0 = debug_text(&sim, h0);
    let text1 = debug_text(&sim, h1);

    // Ship 0 should see its own position (~1000, 2000)
    assert!(text0.contains("POS:1000,2000"), "ship0 position wrong: {text0}");
    assert!(text0.contains("VEL:10,20"), "ship0 velocity wrong: {text0}");
    assert!(text0.contains("HDG:1.00"), "ship0 heading wrong: {text0}");

    // Ship 1 should see its own position (~-500, -800), NOT ship 0's
    assert!(text1.contains("POS:-500,-800"), "ship1 position wrong: {text1}");
    assert!(text1.contains("VEL:-5,-8"), "ship1 velocity wrong: {text1}");
    // Heading -2.0 is normalized to -2.0 + TAU ≈ 4.28 by the physics engine
    assert!(text1.contains("HDG:4.28"), "ship1 heading wrong: {text1}");
}

// ===========================================================================
// 3. Multi-ship firing (adapts bullet_test::test_hit)
//
// A native ship fires at an enemy. Verifies that fire() works through
// the native controller and that bullets hit the target.
// ===========================================================================

struct FireShip {}

impl NativeShip for FireShip {
    fn tick(&mut self) {
        oort_api::prelude::fire(0);
    }
}

struct DoNothingShip {}

impl NativeShip for DoNothingShip {
    fn tick(&mut self) {}
}

#[test]
#[serial]
fn test_native_fire_hit() {
    let attacker_factory: simulation::NativeShipFactory = Arc::new(|| Box::new(FireShip {}));
    let target_factory: simulation::NativeShipFactory = Arc::new(|| Box::new(DoNothingShip {}));

    let mut sim = simulation::Simulation::new(
        "test",
        0,
        &[Code::Native(attacker_factory), Code::Native(target_factory)],
    );

    let ship0 = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![0.0, 0.0],
        0.0, // facing right toward target
        fighter(0),
    );
    let ship1 = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(1),
    );

    let initial_health = sim.ship(ship1).data().health;

    // Run enough ticks for bullets to travel 200m at 1000m/s (~12 ticks)
    // plus a few firing cycles, but not enough to destroy the 100HP target.
    for _ in 0..30 {
        sim.step();
    }

    assert!(sim.ships.contains(ship0), "attacker should still exist");
    assert!(sim.ships.contains(ship1), "target should still exist after 30 ticks");
    assert_ne!(
        sim.ship(ship1).data().health,
        initial_health,
        "target should have taken damage"
    );
}

// ===========================================================================
// 4. Multi-ship radar (adapts ability_test::test_decoy radar scan pattern)
//
// Ship on team 0 scans and detects an enemy ship on team 1.
// Verifies radar works through native controller state management.
// ===========================================================================

struct RadarShip {}

impl NativeShip for RadarShip {
    fn tick(&mut self) {
        use oort_api::prelude::*;
        set_radar_heading(0.0);
        set_radar_width(std::f64::consts::TAU);
        if let Some(contact) = scan() {
            debug!("CONTACT:{:.0},{:.0}", contact.position.x, contact.position.y);
        } else {
            debug!("NO_CONTACT");
        }
    }
}

#[test]
#[serial]
fn test_native_radar() {
    let scanner_factory: simulation::NativeShipFactory = Arc::new(|| Box::new(RadarShip {}));
    let target_factory: simulation::NativeShipFactory = Arc::new(|| Box::new(DoNothingShip {}));

    let mut sim = simulation::Simulation::new(
        "test",
        0,
        &[Code::Native(scanner_factory), Code::Native(target_factory)],
    );

    ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    let _target = ship::create(
        &mut sim,
        vector![500.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(1),
    );

    // Radar results lag by one tick; step twice.
    sim.step();
    sim.step();

    let events = sim.events();
    let texts: Vec<&String> = events.debug_text.values().collect();
    let found_contact = texts.iter().any(|t| t.contains("CONTACT:500,0"));
    assert!(found_contact, "should detect enemy at (500,0), got: {:?}", texts);
}

// ===========================================================================
// 5. Multi-tick stability
//
// Runs 3 ships for 100 ticks. Each ship accumulates a tick counter and
// reports it. Verifies state persists correctly across many ticks and
// that no ship crashes.
// ===========================================================================

struct CountingShip {
    count: u32,
    my_id: u32,
}

impl NativeShip for CountingShip {
    fn tick(&mut self) {
        self.count += 1;
        // Verify our ID stays stable (state not leaking between ships)
        let current_id = oort_api::prelude::id();
        assert_eq!(
            current_id, self.my_id,
            "ID changed! expected {} got {}",
            self.my_id, current_id
        );
        oort_api::prelude::debug!("TICK:{} ID:{}", self.count, self.my_id);
    }
}

#[test]
#[serial]
fn test_native_multi_tick_stability() {
    let factory: simulation::NativeShipFactory = Arc::new(|| {
        let id = oort_api::prelude::id();
        Box::new(CountingShip { count: 0, my_id: id })
    });
    let code = Code::Native(factory);

    let mut sim = simulation::Simulation::new("test", 0, &[code, Code::None]);

    let h0 = ship::create(&mut sim, vector![0.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));
    let h1 = ship::create(&mut sim, vector![100.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));
    let h2 = ship::create(&mut sim, vector![200.0, 0.0], vector![0.0, 0.0], 0.0, fighter(0));

    for _ in 0..100 {
        sim.step();
    }

    // All ships should still exist (no crashes from state corruption)
    assert!(sim.ships.contains(h0), "ship0 should still exist");
    assert!(sim.ships.contains(h1), "ship1 should still exist");
    assert!(sim.ships.contains(h2), "ship2 should still exist");

    // Each ship should report tick count of 100
    let text0 = debug_text(&sim, h0);
    let text1 = debug_text(&sim, h1);
    let text2 = debug_text(&sim, h2);
    assert!(text0.contains("TICK:100"), "ship0 should be at tick 100, got: {text0}");
    assert!(text1.contains("TICK:100"), "ship1 should be at tick 100, got: {text1}");
    assert!(text2.contains("TICK:100"), "ship2 should be at tick 100, got: {text2}");

    // No errors
    assert!(sim.events().errors.is_empty(), "should have no errors");
}

// ===========================================================================
// 6. Acceleration via native controller
//
// A ship calls accelerate() and we verify it actually moves. This tests
// that apply_system_state correctly reads back the acceleration commands.
// ===========================================================================

struct AccelerateShip {}

impl NativeShip for AccelerateShip {
    fn tick(&mut self) {
        oort_api::prelude::accelerate(oort_api::prelude::vec2(100.0, 0.0));
    }
}

#[test]
#[serial]
fn test_native_acceleration() {
    let factory: simulation::NativeShipFactory = Arc::new(|| Box::new(AccelerateShip {}));
    let code = Code::Native(factory);

    let mut sim = simulation::Simulation::new("test", 0, &[code, Code::None]);

    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );

    // Step 1: AI sets acceleration. Step 2: physics applies the force.
    sim.step();
    sim.step();

    let v = sim.ship(ship0).velocity();
    assert!(v.x > 0.0, "ship should have positive x velocity after accelerating, got {}", v.x);

    for _ in 0..58 {
        sim.step();
    }

    let pos = sim.ship(ship0).position();
    assert!(pos.x > 0.0, "ship should have moved right, got x={}", pos.x);
}

// ===========================================================================
// 7. Mixed native + WASM teams (adapts collision_test::test_missile_fighter_collision_different_team)
//
// Team 0 is native, team 1 uses Code::None (manual control). Verifies
// that native and non-native teams coexist in the same simulation.
// ===========================================================================

#[test]
#[serial]
fn test_native_mixed_teams() {
    let factory: simulation::NativeShipFactory = Arc::new(|| Box::new(FireShip {}));

    let mut sim = simulation::Simulation::new(
        "test",
        0,
        &[Code::Native(factory), Code::None],
    );

    let attacker = ship::create(
        &mut sim,
        vector![-100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    let target = ship::create(
        &mut sim,
        vector![100.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        ship::target(1),
    );

    // Target has 1 HP, should be destroyed quickly by vulcan fire
    for _ in 0..200 {
        sim.step();
        if !sim.ships.contains(target) {
            break;
        }
    }

    assert!(sim.ships.contains(attacker), "attacker should survive");
    assert!(!sim.ships.contains(target), "target should be destroyed");
}
