#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use qcell::LCellOwner;
use socket_server::{app::socket_listener::Container, socket_server::entry_point};

fn main() {
    run_with_stack_size(64 * 1024 * 1024, || {
        let addr = "[::]:25555".parse().unwrap();
        LCellOwner::scope(|mut owner| {
            entry_point(&mut owner, Container::default(), addr);
        })
    });
}

fn run_with_stack_size<F>(stack_size: usize, f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(stack_size)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}
