use criterion::Criterion;

fn benchmark(c: &mut Criterion) {
    c.bench_function("test", |b| {
        b.iter(|| {});
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
