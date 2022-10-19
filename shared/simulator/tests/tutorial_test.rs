use oort_simulator::scenario;
use oort_simulator::simulation;
use rayon::prelude::*;
use std::time::Instant;
use test_log::test;

fn check_solution(scenario_name: &str) {
    (0..10u32).into_par_iter().for_each(|seed| {
        let start_time = Instant::now();
        let check_once = |seed: u32| -> u64 {
            let scenario = scenario::load(scenario_name);
            let mut codes = scenario.initial_code();
            codes[0] = scenario.solution().clone();
            let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);

            let mut i = 0;
            while sim.status() == scenario::Status::Running && i < 10000 {
                sim.step();
                i += 1;
            }

            assert_eq!(
                sim.status(),
                scenario::Status::Victory { team: 0 },
                "tutorial {} did not succeed with seed {}",
                scenario_name,
                seed
            );
            sim.hash()
        };
        let hashes: Vec<u64> = (0..2usize)
            .into_par_iter()
            .map(|_| check_once(seed))
            .collect();
        assert_eq!(
            hashes[0], hashes[1],
            "tutorial {} was not deterministic",
            scenario_name
        );
        log::info!(
            "{} seed {} took {:?}",
            scenario_name,
            seed,
            Instant::now() - start_time
        );
    });
}

#[test]
fn test_tutorials() {
    let scenario_names: Vec<String> = scenario::list()
        .iter()
        .filter(|x| x.starts_with("tutorial"))
        .cloned()
        .collect();
    scenario_names
        .into_par_iter()
        .for_each(|x| check_solution(&x));
}

#[test]
fn test_gunnery() {
    check_solution("gunnery");
}

#[test]
fn test_missiles() {
    check_solution("missile_test");
}
