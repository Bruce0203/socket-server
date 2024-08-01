#![feature(core_intrinsics)]
#![feature(variant_count)]

use std::{hint::black_box, intrinsics::transmute_unchecked, mem::variant_count};

use criterion::Criterion;
use rand::Rng;

fn benchmark(c: &mut Criterion) {
    let random: u8 = rand::thread_rng().gen_range(0..variant_count::<A>()) as u8;
    #[repr(u8)]
    enum A {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
    }
    let value: A = unsafe { transmute_unchecked(random) };

    c.bench_function("t", |b| {
        b.iter(|| {
            let result: usize = match value {
                A::A => 981459871,
                A::B => 21348989715,
                A::C => 18274589172498,
                A::D => 28475198245,
                A::E => 82347598748,
                A::F => 81789234579,
                A::G => 8485972849,
                A::H => 192845912459,
                A::I => 1289451792,
                A::J => 981247589172,
                A::K => 1023581839,
            };
            black_box(result);
        });
    });
}

criterion::criterion_main!(benches);
criterion::criterion_group!(benches, benchmark);
