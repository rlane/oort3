use oort_simulator::scenario;
use oort_simulator::simulation;
use rayon::prelude::*;
use test_log::test;

fn check_solution(scenario_name: &str) {
    let check_once = || -> u64 {
        let scenario = scenario::load(scenario_name);
        let mut sim = simulation::Simulation::new(scenario_name, 0, &scenario.solution());

        let mut i = 0;
        while sim.status() == scenario::Status::Running && i < 10000 {
            sim.step();
            i += 1;
        }

        assert_eq!(
            sim.status(),
            scenario::Status::Victory { team: 0 },
            "tutorial {} did not succeed",
            scenario_name
        );
        sim.hash()
    };
    let hashes: Vec<u64> = (0..2usize).into_par_iter().map(|_| check_once()).collect();
    assert_eq!(
        hashes[0], hashes[1],
        "tutorial {} was not deterministic",
        scenario_name
    );
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
