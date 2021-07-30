use oort::simulation;
use oort::simulation::scenario;
use rayon::prelude::*;
use test_env_log::test;

fn check_solution(scenario_name: &str) {
    let check_once = || -> u64 {
        let mut sim = simulation::Simulation::new();
        let mut scenario = scenario::load(scenario_name);
        sim.upload_code(/*team=*/ 0, &scenario.solution());
        scenario.init(&mut sim, 0);

        let mut i = 0;
        while scenario.status(&sim) == scenario::Status::Running && i < 10000 {
            scenario.tick(&mut sim);
            sim.step();
            i += 1;
        }

        assert_eq!(scenario.status(&sim), scenario::Status::Finished);
        sim.hash()
    };
    let hashes: Vec<u64> = (0..2usize).into_par_iter().map(|_| check_once()).collect();
    assert_eq!(hashes[0], hashes[1]);
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
