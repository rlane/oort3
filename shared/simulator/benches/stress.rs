use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation;

fn stress() {
    let scenario = scenario::load("stress");
    let mut sim = simulation::Simulation::new("stress", 0, &scenario.solution_codes());
    while sim.status() == scenario::Status::Running && sim.tick() < 60 * 3 {
        sim.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stress", |b| b.iter(stress));
}

pub fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(core::time::Duration::from_secs(10))
}

criterion_group!(name = benches;
                 config = criterion_config();
                 targets = criterion_benchmark);
criterion_main!(benches);
