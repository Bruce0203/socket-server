use std::{
    mem::{transmute_copy, MaybeUninit},
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

use fast_collections::{Cursor, Slab, Vec};
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use qcell::{LCell, LCellOwner};

use crate::tick_machine::TickMachine;

use super::socket::{Registry, ServerSocketListener, Socket, SocketState};

pub struct ServerSelector<'id, 'registry, T: ServerSocketListener<'id>>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    server: T,
    selector: Selector<'id, 'registry, T>,
    poll: Poll,
    mio_registry: mio::Registry,
}

pub struct Selector<'id, 'registry, T: ServerSocketListener<'id>>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    sockets: Slab<Socket<'id, 'registry, T>, { T::MAX_CONNECTIONS }>,
    streams: [MaybeUninit<TcpStream>; T::MAX_CONNECTIONS],
}

impl<'id, 'registry, T> Selector<'id, 'registry, T>
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn new() -> Self {
        let streams = MaybeUninit::<[MaybeUninit<TcpStream>; T::MAX_CONNECTIONS]>::uninit();
        let streams = unsafe { transmute_copy(&streams.assume_init()) };
        Self {
            sockets: Slab::new(),
            streams,
        }
    }
}

impl<'id, 'registry, T> ServerSelector<'id, 'registry, T>
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    fn new(server: T) -> Self {
        let poll = Poll::new().unwrap();
        let mio_registry = poll.registry().try_clone().unwrap();
        Self {
            poll,
            mio_registry,
            selector: Selector::new(),
            server,
        }
    }

    fn accept(
        &mut self,
        owner: &mut LCellOwner<'id>,
        listener: &mut TcpListener,
        registry: &'registry LCell<'id, Registry<'id, T>>,
    ) -> Result<(), ()> {
        let (accepted_stream, _addr) = listener.accept().map_err(|_| ())?;
        let id = self.selector.sockets.add_with_index(|ind| Socket {
            connection: Default::default(),
            state: SocketState::default(),
            read_buf: owner.cell(Cursor::new()),
            write_buf: owner.cell(Cursor::new()),
            token: *ind,
            registry,
        })?;
        let stream = unsafe { self.selector.streams.get_unchecked_mut(id) };
        let socket = unsafe { self.selector.sockets.get_unchecked_mut(id) };
        *stream = MaybeUninit::new(accepted_stream);
        match mio::Registry::register(
            &self.mio_registry,
            unsafe { stream.assume_init_mut() },
            Token(socket.token),
            Interest::READABLE,
        ) {
            Ok(()) => self.server.accept(owner, socket),
            Err(_err) => socket.register_close_event(owner),
        }
        Ok(())
    }

    fn read(&mut self, owner: &mut LCellOwner<'id>, token: usize) {
        let socket = unsafe { self.selector.sockets.get_unchecked_mut(token) };
        let stream = unsafe {
            self.selector
                .streams
                .get_unchecked_mut(token)
                .assume_init_mut()
        };
        match socket.read_buf.rw(owner).push_from_read(stream) {
            Ok(()) => self.server.read(owner, socket),
            Err(_) => socket.register_close_event(owner),
        }
    }

    fn flush_registry(
        &mut self,
        owner: &mut LCellOwner<'id>,
        registry: &'registry LCell<'id, Registry<'id, T>>,
    ) {
        let registry_vec_len = registry.ro(owner).len();
        for ind in 0..registry_vec_len {
            let id = *unsafe { registry.ro(&owner).get_unchecked(ind) };
            let socket = unsafe { self.selector.sockets.get_unchecked_mut(id) };
            let stream = unsafe {
                self.selector
                    .streams
                    .get_unchecked_mut(id)
                    .assume_init_mut()
            };
            match socket.state {
                SocketState::Idle => continue,
                SocketState::WriteRequest => {
                    socket.state = SocketState::Idle;
                    self.server.flush(owner, socket);
                    match socket.write_buf.rw(owner).push_to_write(stream) {
                        Ok(()) => {}
                        Err(_) => self.close(owner, id),
                    };
                }
                SocketState::CloseRequest => self.close(owner, id),
            }
        }
        registry.rw(owner).clear();
    }

    fn close(&mut self, owner: &mut LCellOwner<'id>, id: usize) {
        let socket = unsafe { self.selector.sockets.get_unchecked_mut(id) };
        let stream = unsafe {
            self.selector
                .streams
                .get_unchecked_mut(id)
                .assume_init_mut()
        };
        self.server.close(owner, socket);
        self.mio_registry.deregister(stream).unwrap();
        let token = socket.token;
        unsafe { self.selector.sockets.remove_unchecked(token) };
    }
}

pub fn listen<'id, T>(owner: &mut LCellOwner<'id>, server: T, addr: impl ToSocketAddrs) -> !
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    let registry = owner.cell(Registry { vec: Vec::uninit() });
    let mut selector = ServerSelector::new(server);
    const LISTENER_TOKEN: Token = Token(usize::MAX);
    let addr = addr.to_socket_addrs().unwrap().next().unwrap();
    let mut listener = {
        let mut listener = TcpListener::bind(addr).unwrap();
        selector
            .mio_registry
            .register(&mut listener, LISTENER_TOKEN, Interest::READABLE)
            .unwrap();
        listener
    };
    let mut events = Events::with_capacity(T::MAX_CONNECTIONS);
    let mut tick_machine = TickMachine::new(T::TICK);
    loop {
        selector
            .poll
            .poll(&mut events, Some(Duration::ZERO))
            .unwrap();
        tick_machine.tick(|| selector.server.tick(owner));
        for event in events.iter() {
            let token = event.token();
            if token == LISTENER_TOKEN {
                let _result = selector.accept(owner, &mut listener, &registry);
            } else {
                selector.read(owner, token.0)
            }
        }
        selector.flush_registry(owner, &registry)
    }
}
