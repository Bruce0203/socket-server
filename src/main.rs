#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use qcell::LCellOwner;
use socket_server::{app::app::App, net::selector::listen};
use std::{io::Result, thread::JoinHandle};

fn main() {
    entry_point()
}

pub fn entry_point() {
    run_with_stack_size(64 * 1024 * 1024, || {
        let addr = "[::]:25555".parse().unwrap();
        let app = App::default();
        LCellOwner::scope(|mut owner| listen(&mut owner, app, addr))
    })
    .unwrap()
    .join()
    .unwrap()
}

fn run_with_stack_size<F>(stack_size: usize, f: F) -> Result<JoinHandle<()>>
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new().stack_size(stack_size).spawn(f)
}

#[test]
fn test_server() {}
