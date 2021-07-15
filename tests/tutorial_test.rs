use oort::simulation;
use oort::simulation::scenario;

fn check_solution(scenario_name: &str) {
    let mut sim = simulation::Simulation::new();
    let scenario = scenario::load(scenario_name);
    scenario.init(&mut sim);
    sim.upload_code(&scenario.solution());

    for _ in 0..10000 {
        sim.step();
        if scenario.tick(&mut sim) == scenario::Status::Finished {
            break;
        }
    }

    assert_eq!(scenario.tick(&mut sim), scenario::Status::Finished);
}

#[test]
fn test_tutorial01() {
    check_solution("tutorial01");
}

#[test]
fn test_tutorial02() {
    check_solution("tutorial02");
}
