use std::hint::black_box;

use criterion::Criterion;
use mio::Poll;
use rand::Rng;

fn benchmark(c: &mut Criterion) {
    let value: u64 = rand::thread_rng().gen();
    println!("{value:?}");
    let poll = Poll::new().unwrap();
    let registry = poll.registry();
    c.bench_function("t", |b| {
        b.iter(|| {
            black_box(&registry.try_clone().unwrap());
        });
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
