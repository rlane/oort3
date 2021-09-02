use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::simulation;
use oort_simulator::simulation::scenario;

fn check_solution(scenario_name: &str) {
    let scenario = scenario::load(scenario_name);
    let mut sim = simulation::Simulation::new(scenario_name, 0, &scenario.solution());

    let mut i = 0;
    while sim.status() == scenario::Status::Running && i < 10000 {
        sim.step();
        i += 1;
    }

    assert_ne!(sim.status(), scenario::Status::Running);
}

fn tutorials() {
    for scenario_name in &[
        "tutorial01",
        "tutorial02",
        "tutorial03",
        "tutorial04",
        "tutorial05",
        "tutorial06",
        "tutorial07",
    ] {
        check_solution(scenario_name);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tutorials", |b| b.iter(|| tutorials()));
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
