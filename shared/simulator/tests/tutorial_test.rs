use oort_simulator::scenario;
use oort_simulator::simulation;
use rayon::prelude::*;
use std::time::Instant;
use test_log::test;

fn check_solution(scenario_name: &str, seed: u32) {
    let start_time = Instant::now();
    let check_once = |seed: u32| -> u64 {
        let scenario = scenario::load(scenario_name);
        let mut codes = scenario.initial_code();
        codes[0] = scenario.solution();
        let mut sim = simulation::Simulation::new(scenario_name, seed, &codes);

        let mut i = 0;
        while sim.status() == scenario::Status::Running && i < 10000 {
            sim.step();
            i += 1;
        }

        assert_eq!(
            sim.status(),
            scenario::Status::Victory { team: 0 },
            "tutorial {scenario_name} did not succeed with seed {seed}"
        );
        sim.hash()
    };
    let hashes: Vec<u64> = (0..2usize)
        .map(|_| check_once(seed))
        .collect();
    assert_eq!(
        hashes[0], hashes[1],
        "tutorial {scenario_name} was not deterministic"
    );
    log::info!(
        "{} seed {} took {:?}",
        scenario_name,
        seed,
        Instant::now() - start_time
    );
}

#[test]
fn test_tutorials() {
    let categories = scenario::list();
    let scenario_names: &Vec<String> = &categories
        .iter()
        .find(|(category, _)| category == "Tutorial")
        .unwrap()
        .1;
    assert!(!scenario_names.is_empty());

    let cases: Vec<(&str, u32)> = scenario_names
        .iter()
        .flat_map(|name| (0..10).map(move |seed| (name.as_str(), seed)))
        .collect();

    cases
        .into_par_iter()
        .for_each(|(name, seed)| check_solution(name, seed));
}

#[test]
fn test_gunnery() {
    (0..10u32).into_par_iter().for_each(|seed| {
        check_solution("gunnery", seed);
    });
}

#[test]
fn test_missiles() {
    (0..10u32).into_par_iter().for_each(|seed| {
        check_solution("missile_test", seed);
    });
}

#[test]
fn test_welcome() {
    let scenario_name = "welcome";
    let scenario = scenario::load(scenario_name);
    let mut codes = scenario.initial_code();
    codes[0] = scenario.solution();
    let mut sim = simulation::Simulation::new(scenario_name, 0, &codes);

    let mut i = 0;
    while sim.status() == scenario::Status::Running && i < scenario::MAX_TICKS {
        sim.step();
        i += 1;
    }

    assert_eq!(sim.status(), scenario::Status::Running);
}
