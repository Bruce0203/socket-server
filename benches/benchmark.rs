use std::{
    hint::black_box,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    thread::sleep,
    time::Duration,
};

use criterion::Criterion;
use polling::{Event, Poller};

fn benchmark(c: &mut Criterion) {
    c.bench_function("t", |b| {
        b.iter(|| {
            loop_code::repeat!(i 500 {
                black_box(i + 1);
            });
        });
    });
    c.bench_function("t2", |b| {
        b.iter(|| {
            for i in 0..500 {
                black_box(i + 1);
            }
        })
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
