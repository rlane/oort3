use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation;

fn check_solution(scenario_name: &str) {
    let scenario = scenario::load(scenario_name);
    let mut sim = simulation::Simulation::new(scenario_name, 0, &scenario.solution_codes());

    let mut i = 0;
    while sim.status() == scenario::Status::Running && i < 10000 {
        sim.step();
        i += 1;
    }

    assert_ne!(sim.status(), scenario::Status::Running);
}

fn tutorials() {
    let categories = scenario::list();
    let scenario_names: &Vec<String> = &categories
        .iter()
        .find(|(category, _)| category == "Tutorial")
        .unwrap()
        .1;
    assert!(!scenario_names.is_empty());
    for scenario_name in scenario_names {
        check_solution(scenario_name);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tutorials", |b| b.iter(tutorials));
}

pub fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(core::time::Duration::from_secs(20))
}

criterion_group!(name = benches;
                 config = criterion_config();
                 targets = criterion_benchmark);
criterion_main!(benches);
