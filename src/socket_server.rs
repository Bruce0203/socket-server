use crate::tick_machine::TickMachine;
use derive_more::{Deref, DerefMut};
use fast_collections::{Cursor, Slab, Vec};
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use qcell::{LCell, LCellOwner};
use std::{net::SocketAddr, time::Duration, usize};

#[derive(Deref, DerefMut)]
pub struct Socket<'id: 'registry, 'registry, T: SocketListener<'id>>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    #[deref]
    #[deref_mut]
    pub connection: T::Connection,
    pub read_buf: LCell<'id, Cursor<u8, { T::READ_BUFFFER_LEN }>>,
    pub write_buf: LCell<'id, Cursor<u8, { T::WRITE_BUFFER_LEN }>>,
    stream: TcpStream,
    state: SocketState,
    token: usize,
    registry: &'registry LCell<'id, Registry<'id, T>>,
}

#[derive(Deref, DerefMut)]
pub(self) struct Registry<'id, T: SocketListener<'id>>
where
    [(); T::MAX_CONNECTIONS]:,
{
    vec: Vec<usize, { T::MAX_CONNECTIONS }>,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SocketState {
    #[default]
    Idle,
    WriteRequest,
    CloseRequest,
}

impl<'id, 'registry, T: SocketListener<'id>> Socket<'id, 'registry, T>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn register_flush_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::WriteRequest;
    }

    pub fn register_close_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::CloseRequest;
    }

    pub(self) fn register_event(&mut self, owner: &mut LCellOwner<'id>) {
        if self.state == SocketState::Idle {
            let registry = owner.rw(self.registry);
            unsafe { registry.push_unchecked(self.token) };
        }
    }
}

pub trait SocketListener<'id>: Sized {
    const MAX_CONNECTIONS: usize;
    const READ_BUFFFER_LEN: usize;
    const WRITE_BUFFER_LEN: usize;
    const TICK: Duration;
    type Connection;

    fn tick(&mut self, owner: &mut LCellOwner<'id>);

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn read(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn flush(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn close(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;
}

pub struct Selector<'id, 'registry, T: SocketListener<'id>>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    poll: Poll,
    mio_registry: mio::Registry,
    sockets: Slab<Socket<'id, 'registry, T>, { T::MAX_CONNECTIONS }>,
    server: T,
}

impl<'id, 'registry, T> Selector<'id, 'registry, T>
where
    T: SocketListener<'id, Connection: Default>,
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
            sockets: Slab::new(),
            server,
        }
    }

    fn accept(
        &mut self,
        owner: &mut LCellOwner<'id>,
        listener: &mut TcpListener,
        registry: &'registry LCell<'id, Registry<'id, T>>,
    ) -> Result<(), ()> {
        let (stream, _addr) = listener.accept().map_err(|_| ())?;
        let id = self.sockets.add_with_index(|ind| Socket::<'_, '_, T> {
            connection: T::Connection::default(),
            stream,
            state: SocketState::default(),
            read_buf: owner.cell(Cursor::new()),
            write_buf: owner.cell(Cursor::new()),
            token: *ind,
            registry,
        })?;
        let socket = unsafe { self.sockets.get_unchecked_mut(id) };
        match mio::Registry::register(
            &self.mio_registry,
            &mut socket.stream,
            Token(socket.token),
            Interest::READABLE,
        ) {
            Ok(()) => self.server.accept(owner, socket),
            Err(_) => socket.register_close_event(owner),
        }
        Ok(())
    }

    fn read(&mut self, owner: &mut LCellOwner<'id>, token: usize) {
        let socket = unsafe { self.sockets.get_unchecked_mut(token) };
        match socket.read_buf.rw(owner).push_from_read(&mut socket.stream) {
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
            let socket = unsafe { self.sockets.get_unchecked_mut(id) };
            match socket.state {
                SocketState::Idle => continue,
                SocketState::WriteRequest => {
                    socket.state = SocketState::Idle;
                    self.server.flush(owner, socket);
                    match socket.write_buf.rw(owner).push_to_write(&mut socket.stream) {
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
        let socket = unsafe { self.sockets.get_unchecked_mut(id) };
        self.server.close(owner, socket);
        self.mio_registry.deregister(&mut socket.stream).unwrap();
        let token = socket.token;
        unsafe { self.sockets.remove_unchecked(token) };
    }
}

pub fn entry_point<'id, T>(owner: &mut LCellOwner<'id>, server: T, addr: SocketAddr)
where
    T: SocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    let registry = owner.cell(Registry { vec: Vec::uninit() });
    let mut selector = Selector::new(server);
    const LISTENER_TOKEN: Token = Token(usize::MAX);
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
