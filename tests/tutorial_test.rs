use oort::simulation;
use oort::simulation::scenario;
use test_env_log::test;

fn check_solution(scenario_name: &str) {
    let mut sim = simulation::Simulation::new();
    let mut scenario = scenario::load(scenario_name);
    scenario.init(&mut sim);
    sim.upload_code(&scenario.solution(), /*team=*/ 0);

    let mut i = 0;
    while scenario.status(&sim) == scenario::Status::Running && i < 10000 {
        scenario.tick(&mut sim);
        sim.step();
        i += 1;
    }

    assert_eq!(scenario.status(&sim), scenario::Status::Finished);
}

#[test]
fn test_tutorial01() {
    check_solution("tutorial01");
}

#[test]
fn test_tutorial02() {
    check_solution("tutorial02");
}

#[test]
fn test_tutorial03() {
    check_solution("tutorial03");
}

#[test]
fn test_tutorial04() {
    check_solution("tutorial04");
}

#[test]
fn test_tutorial05() {
    check_solution("tutorial05");
}
