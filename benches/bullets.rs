use criterion::{criterion_group, criterion_main, Criterion};
use oort::simulation;
use oort::simulation::scenario;

fn many_bullets() {
    let mut sim = simulation::Simulation::new();
    let mut scenario = scenario::load("bullet-stress");
    scenario.init(&mut sim, 0);

    while scenario.status(&sim) == scenario::Status::Running {
        scenario.tick(&mut sim);
        sim.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("many_bullets", |b| b.iter(|| many_bullets()));
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
