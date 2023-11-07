use oort_simulator::scenario;
use oort_simulator::simulation;
use test_log::test;

fn run(scenario_name: &str) -> u64 {
    let scenario = scenario::load(scenario_name);
    let codes = scenario.solution_codes();
    let seed = 0;
    let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);

    while sim.status() == scenario::Status::Running {
        sim.step();
    }

    sim.hash()
}

#[test]
fn test_frigate_vs_cruiser() {
    assert_eq!(run("frigate_vs_cruiser"), 8487625765679226385);
}
