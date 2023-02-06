use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oort_compiler::Compiler;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("compile without reuse", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            black_box(compiler.compile(include_str!("../../ai/empty.rs")).unwrap());
        })
    });

    c.bench_function("compile with reuse", |b| {
        let mut compiler = Compiler::new();
        black_box(compiler.compile(include_str!("../../ai/empty.rs")).unwrap());
        b.iter(|| {
            black_box(compiler.compile(include_str!("../../ai/empty.rs")).unwrap());
        })
    });
}

pub fn criterion_config() -> Criterion {
    Criterion::default().sample_size(10)
}

criterion_group!(name = benches;
                 config = criterion_config();
                 targets = criterion_benchmark);
criterion_main!(benches);
