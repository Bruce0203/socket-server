use std::{io::Write, thread::sleep_ms};

use criterion::Criterion;
use mio::{
    net::{TcpListener, TcpStream},
    Interest, Poll, Token,
};

fn benchmark(c: &mut Criterion) {
    let mut poll = Poll::new().unwrap();
    let registry = poll.registry();
    let listener = TcpListener::bind("0.0.0.0:0".parse().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    println!("{}", addr);
    let mut stream = TcpStream::connect(addr.parse().unwrap()).unwrap();
    sleep_ms(100);
    registry
        .register(&mut stream, Token(0), Interest::WRITABLE)
        .unwrap();
    c.bench_function("test", |b| {
        b.iter(|| {
            stream.write(&[]).unwrap();
        });
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
