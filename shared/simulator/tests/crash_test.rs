use nalgebra::vector;
use oort_simulator::ship::{self, fighter};
use oort_simulator::simulation::{self, Code};
use serial_test::serial;
use std::collections::BTreeMap;

#[test]
#[serial]
fn test_panic() {
    let mut sim =
        simulation::Simulation::new("test", 0, &[Code::Builtin("test".to_string()), Code::None]);
    let mut env = BTreeMap::new();
    env.insert("TESTCASE".to_string(), "panic".to_string());
    sim.update_environment(0, env);
    ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );

    testing_logger::setup();
    sim.step();
    testing_logger::validate(|captured_logs| {
        assert_eq!(captured_logs.len(), 1);
        assert_eq!(captured_logs[0].level, log::Level::Warn);
        assert_eq!(&captured_logs[0].body, "ship panicked at 'Panic!', lib.rs:17:24");
    });
}

#[test]
#[serial]
fn test_infinite_loop() {
    let mut sim =
        simulation::Simulation::new("test", 0, &[Code::Builtin("test".to_string()), Code::None]);
    let mut env = BTreeMap::new();
    env.insert("TESTCASE".to_string(), "infinite_loop".to_string());
    sim.update_environment(0, env);
    ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );

    testing_logger::setup();
    sim.step();
    testing_logger::validate(|captured_logs| {
        assert_eq!(captured_logs.len(), 1);
        assert_eq!(captured_logs[0].level, log::Level::Warn);
        assert_eq!(
            captured_logs[0].body,
            "Ship exceeded maximum number of instructions and was destroyed"
        );
    });
}
