use criterion::Criterion;
use mio::{net::TcpStream, Interest, Poll, Token};

fn benchmark(c: &mut Criterion) {
    let mut poll = Poll::new().unwrap();
    let registry = poll.registry();
    let mut stream = TcpStream::connect("0.0.0.0:1234".parse().unwrap()).unwrap();
    registry
        .register(&mut stream, Token(0), Interest::WRITABLE)
        .unwrap();
    c.bench_function("test", |b| {
        b.iter(|| {
            registry
                .reregister(&mut stream, Token(0), Interest::WRITABLE)
                .unwrap();
        });
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
