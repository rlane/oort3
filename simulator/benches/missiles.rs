use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation::{self, Code};

fn missile_stress() {
    let mut sim = simulation::Simulation::new("missile-stress", 0, &Code::None);
    while sim.status() == scenario::Status::Running {
        sim.step();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("missile-stress", |b| b.iter(|| missile_stress()));
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
