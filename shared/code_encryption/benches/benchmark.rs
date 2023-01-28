use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encryption", |b| {
        b.iter(|| oort_code_encryption::encrypt(black_box("foo")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
