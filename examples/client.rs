#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{io::Write, net::SocketAddr, thread::sleep, time::Duration};

use fast_collections::Vec;
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};

fn main() {
    const MAX_CONNECTIONS: usize = 100_000;
    let mut events = Events::with_capacity(MAX_CONNECTIONS);
    let mut poll = Poll::new().unwrap();

    let mut token_acc = 0;
    let target_addr: SocketAddr = "158.180.88.171:25555".parse().unwrap();

    let mut connections = Vec::<(usize, TcpStream), MAX_CONNECTIONS>::uninit();
    loop {
        let mut stream = TcpStream::connect(target_addr).unwrap();
        poll.registry()
            .register(&mut stream, Token(token_acc), Interest::READABLE)
            .unwrap();
        connections.push((token_acc, stream)).unwrap();
        token_acc += 1;
        sleep(Duration::from_millis(1));
        println!("{token_acc}");
        poll.poll(&mut events, Some(Duration::ZERO)).unwrap();
        for event in events.iter() {
            println!("READ");
        }
    }
}
