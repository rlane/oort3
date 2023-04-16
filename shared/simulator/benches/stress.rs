use criterion::{criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation;
use oort_simulator::snapshot::Timing;

fn stress(timing: &mut Timing) {
    let scenario = scenario::load("stress");
    let mut sim = simulation::Simulation::new("stress", 0, &scenario.solution_codes());
    while sim.status() == scenario::Status::Running && sim.tick() < 60 * 3 {
        sim.step();
        *timing += sim.timing().clone();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stress", |b| {
        let mut timing = Timing::default();
        let mut c = 0;
        b.iter(|| {
            c += 1;
            stress(&mut timing)
        });
        if timing.total() > 1.0 {
            println!(
                "\nTimings: {:.0?} total: {:.0}ms",
                timing.clone() * (1e3 / c as f64),
                timing.total() * (1e3 / c as f64)
            );
        }
    });
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
