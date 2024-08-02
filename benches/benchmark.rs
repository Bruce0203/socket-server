use std::hint::black_box;

use criterion::Criterion;
use rand::Rng;

fn benchmark(c: &mut Criterion) {
    let value: u64 = rand::thread_rng().gen();
    println!("{value:?}");
    c.bench_function("t", |b| {
        b.iter(|| {
            for i in 0..10 {
                if value == 10 {
                    black_box(true)
                } else {
                    black_box(false)
                };
            }
        });
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
