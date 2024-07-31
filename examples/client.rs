#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::net::SocketAddr;

use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};

fn main() {
    const MAX_CONNECTIONS: usize = 1000;
    let events = Events::with_capacity(MAX_CONNECTIONS);
    let mut poll = Poll::new().unwrap();

    let mut token_acc = 0;
    let target_addr: SocketAddr = "158.180.88.171:25555".parse().unwrap();
    loop {
        let mut stream = TcpStream::connect(target_addr).unwrap();
        poll.registry()
            .register(&mut stream, Token(token_acc), Interest::READABLE)
            .unwrap();
        token_acc += 1;
    }
}
