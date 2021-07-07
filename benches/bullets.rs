use criterion::{criterion_group, criterion_main, Criterion};
use oort::simulation;

fn many_bullets() {
    let mut sim = simulation::Simulation::new();

    let ship0 = sim.add_ship(-100.0, 0.0, 0.0, 0.0, 0.0);
    sim.add_ship(100.0, 0.0, 0.0, 0.0, 0.1);

    for _ in 0..1000 {
        sim.fire_weapon(ship0);
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
