use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oort_simulator::scenario;
use oort_simulator::simulation;
use oort_simulator::snapshot::Snapshot;

fn make_snapshot() -> Snapshot {
    let scenario_name = "tutorial07";
    let scenario = scenario::load(scenario_name);
    let mut sim = simulation::Simulation::new(scenario_name, 0, &scenario.solution_codes());
    for _ in 0..300 {
        sim.step();
    }
    sim.snapshot(0)
}

fn criterion_benchmark(c: &mut Criterion) {
    let snapshot = make_snapshot();
    c.bench_function("json", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<Snapshot>(
                &serde_json::to_string(&snapshot).unwrap(),
            ))
        })
    });
    c.bench_function("bincode", |b| {
        b.iter(|| {
            black_box(bincode::deserialize::<Snapshot>(
                &bincode::serialize(&snapshot).unwrap(),
            ))
        })
    });
}

pub fn criterion_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches;
                 config = criterion_config();
                 targets = criterion_benchmark);
criterion_main!(benches);
