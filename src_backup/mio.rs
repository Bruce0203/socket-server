use std::time::Duration;

use fast_collections::Cursor;
use mio::{event::Source, net::TcpStream, Interest, Registry, Token};

use crate::{socket_id::SocketId, tick_machine::TickMachine, Read, ServerSocket, Stream, Write};

pub fn entry_point<T: Default>(
    mut io: impl ServerSocket<Stream: From<TcpStream>, Registry = Registry>,
    port: u16,
    tick: Duration,
) -> ! {
    let mut events = mio::Events::with_capacity(100);
    const LISTENER_INDEX: usize = usize::MAX;
    let mut poll = mio::Poll::new().unwrap();
    let mut registry = poll.registry().try_clone().unwrap();
    let listener = {
        let addr = format!("[::]:{port}").parse().unwrap();
        let mut listener = mio::net::TcpListener::bind(addr).unwrap();
        let lisetner_token = mio::Token(LISTENER_INDEX);
        let interest = mio::Interest::READABLE;
        mio::event::Source::register(&mut listener, &registry, lisetner_token, interest).unwrap();
        listener
    };
    let mut tick_machine = TickMachine::new(tick);
    loop {
        poll.poll(&mut events, Some(Duration::ZERO)).unwrap();
        tick_machine.tick(|| io.tick(&mut registry).unwrap());
        for event in events.iter() {
            if event.token().0 == LISTENER_INDEX {
                io.accept(listener.accept().unwrap().0.into(), &mut registry)
            } else {
                let socket_id = SocketId::from(event.token().0);
                io.poll_read(&socket_id, &mut registry);
            }
        }
        io.flush_all_sockets(&mut registry);
    }
}

pub trait TcpStreamHolder {
    fn tcp_stream(&self) -> &TcpStream;
    fn tcp_stream_mut(&mut self) -> &mut TcpStream;
}

impl Stream for TcpStream {
    type Error = std::io::Error;
    type Registry = Registry;

    fn open(&mut self, token: usize, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.register(registry, Token(token), Interest::READABLE)
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.deregister(registry)
    }
}

impl Write for TcpStream {
    type Error = std::io::Error;

    fn write<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        buffer.push_to_write(self)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Read for TcpStream {
    type Error = std::io::Error;

    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        buffer.push_from_read(self)
    }
}
