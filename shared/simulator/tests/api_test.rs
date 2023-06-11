use nalgebra::vector;
use oort_simulator::ship::{self, fighter};
use oort_simulator::simulation::{self, Code};
use std::collections::BTreeMap;
use test_log::test;

#[test]
fn test_scenario_name() {
    let mut sim =
        simulation::Simulation::new("test", 0, &[Code::Builtin("test".to_string()), Code::None]);
    let mut env = BTreeMap::new();
    env.insert("TESTCASE".to_string(), "scenario_name".to_string());
    sim.update_environment(0, env);
    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    sim.step();
    let output = sim
        .events()
        .debug_text
        .get(&ship0.into())
        .expect("Missing debug text");
    assert!(output.contains("Scenario: test"), "output: {:?}", output);
}

#[test]
fn test_world_size() {
    let mut sim =
        simulation::Simulation::new("test", 0, &[Code::Builtin("test".to_string()), Code::None]);
    let mut env = BTreeMap::new();
    env.insert("TESTCASE".to_string(), "world_size".to_string());
    sim.update_environment(0, env);
    let ship0 = ship::create(
        &mut sim,
        vector![0.0, 0.0],
        vector![0.0, 0.0],
        0.0,
        fighter(0),
    );
    sim.step();
    let output = sim
        .events()
        .debug_text
        .get(&ship0.into())
        .expect("Missing debug text");
    assert!(
        output.contains("World size: 200000"),
        "output: {:?}",
        output
    );
}
