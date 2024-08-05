use std::{net::ToSocketAddrs, time::Duration};

use qcell::LCellOwner;

use crate::{
    selector::{Poll, Selector},
    socket::{Registry, ServerSocketListener},
    tick_machine::TickMachine,
};

pub(self) struct MioPoll {
    mio_poll: mio::Poll,
    mio_registry: mio::Registry,
}

impl MioPoll {
    pub fn new() -> Self {
        let poll = mio::Poll::new().unwrap();
        let mio_registry = poll.registry().try_clone().unwrap();
        Self {
            mio_poll: poll,
            mio_registry,
        }
    }
}

impl<T: mio::event::Source> Poll<T> for MioPoll {
    fn open(&mut self, stream: &mut T, token: usize) -> Result<(), ()> {
        stream
            .register(
                &self.mio_registry,
                mio::Token(token),
                mio::Interest::READABLE,
            )
            .map_err(|_| ())
    }

    fn close(&mut self, stream: &mut T) {
        let _result = stream.deregister(&self.mio_registry);
    }
}

pub fn listen<'id, T>(
    owner: &mut LCellOwner<'id>,
    server: T,
    addr: impl ToSocketAddrs,
    tick: Duration,
) -> !
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    let registry = owner.cell(Registry::new());
    let mut selector = Selector::<_, _, mio::net::TcpStream>::new(server, owner, MioPoll::new());
    const LISTENER_TOKEN: mio::Token = mio::Token(usize::MAX);
    let addr = addr.to_socket_addrs().unwrap().next().unwrap();
    let listener = {
        let mut listener = mio::net::TcpListener::bind(addr).unwrap();
        selector.poll.open(&mut listener, LISTENER_TOKEN.0).unwrap();
        listener
    };
    let mut events = mio::Events::with_capacity(T::MAX_CONNECTIONS);
    let mut tick_machine = TickMachine::new(tick);
    loop {
        selector
            .poll
            .mio_poll
            .poll(&mut events, Some(Duration::ZERO))
            .unwrap();
        tick_machine.tick(|| T::tick(&selector.server, owner));
        selector.flush_registry(owner, &registry);
        for event in events.iter() {
            let token = event.token();
            if token == LISTENER_TOKEN {
                if let Ok((stream, addr)) = listener.accept() {
                    let _result = selector.accept(owner, stream, addr, &registry);
                }
            } else {
                selector.read(owner, token.0)
            }
        }
    }
}
