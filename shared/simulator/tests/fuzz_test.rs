use oort_simulator::scenario;
use oort_simulator::simulation;
use rayon::prelude::*;
use test_log::test;

#[test]
fn test_fuzz() {
    let scenario_name = "fleet";
    let scenario = scenario::load(scenario_name);
    let mut codes = scenario.initial_code();
    codes[0] = simulation::Code::Builtin("testing/fuzz".to_string());
    (0..10u32).into_par_iter().for_each(|seed| {
        let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);
        let mut i = 0;
        while sim.status() == scenario::Status::Running && i < 200 {
            sim.step();
            i += 1;
        }
    });
}
