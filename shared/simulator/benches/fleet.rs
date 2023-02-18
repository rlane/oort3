use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation;

fn fleet() {
    let scenario = scenario::load("fleet");
    let mut sim = simulation::Simulation::new("fleet", 0, &scenario.solution_codes());
    while sim.status() == scenario::Status::Running {
        sim.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fleet", |b| b.iter(fleet));
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
