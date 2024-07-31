use crate::tick_machine::TickMachine;
use derive_more::{Deref, DerefMut};
use fast_collections::{Slab, Vec};
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use qcell::{LCell, LCellOwner};
use std::{net::SocketAddr, time::Duration, usize};

#[derive(Deref, DerefMut)]
pub struct Socket<'id: 'registry, 'registry, T: SocketListener>
where
    [(); T::MAX_CONNECTIONS]:,
{
    #[deref]
    #[deref_mut]
    connection: T::Connection,
    stream: TcpStream,
    state: SocketState,
    token: usize,
    registry: &'registry LCell<'id, Registry<T>>,
}

#[derive(Deref, DerefMut)]
pub(self) struct Registry<T: SocketListener>
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

impl<'id, 'registry, T: SocketListener> std::io::Write for Socket<'id, 'registry, T>
where
    [(); T::MAX_CONNECTIONS]:,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl<'id, 'registry, T: SocketListener> std::io::Read for Socket<'id, 'registry, T>
where
    [(); T::MAX_CONNECTIONS]:,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<'id, 'registry, T: SocketListener> Socket<'id, 'registry, T>
where
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn register_write_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::WriteRequest;
    }

    pub fn register_close_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::CloseRequest;
    }

    pub fn register_event(&mut self, owner: &mut LCellOwner<'id>) {
        if self.state == SocketState::Idle {
            let registry = owner.rw(self.registry);
            unsafe { registry.push_unchecked(self.token) };
        }
    }
}

pub trait SocketListener: Sized {
    const MAX_CONNECTIONS: usize;
    const TICK: Duration;
    type Connection;

    fn tick<'id>(&mut self, owner: &mut LCellOwner<'id>);

    fn accept<'id>(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::MAX_CONNECTIONS]:;

    fn read<'id>(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::MAX_CONNECTIONS]:;

    fn flush<'id>(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::MAX_CONNECTIONS]:;

    fn close<'id>(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::MAX_CONNECTIONS]:;
}

pub struct Selector<'id, 'registry, T: SocketListener>
where
    [(); T::MAX_CONNECTIONS]:,
{
    poll: Poll,
    mio_registry: mio::Registry,
    sockets: Slab<Socket<'id, 'registry, T>, { T::MAX_CONNECTIONS }>,
    server: T,
}

impl<'id, 'registry, T> Selector<'id, 'registry, T>
where
    T: SocketListener<Connection: Default>,
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
        registry: &'registry LCell<'id, Registry<T>>,
    ) {
        let (mut stream, _addr) = listener.accept().map_err(|_| ()).unwrap();
        let id = self
            .sockets
            .add_with_index(|ind| {
                mio::Registry::register(
                    &self.mio_registry,
                    &mut stream,
                    Token(*ind),
                    Interest::READABLE,
                )
                .unwrap();
                Socket::<'_, '_, T> {
                    stream,
                    registry,
                    connection: T::Connection::default(),
                    state: SocketState::default(),
                    token: *ind,
                }
            })
            .unwrap();
        let socket = unsafe { self.sockets.get_unchecked_mut(id) };
        self.server.accept(owner, socket);
    }

    fn flush_registry(
        &mut self,
        owner: &mut LCellOwner<'id>,
        registry: &'registry LCell<'id, Registry<T>>,
    ) {
        let registry_vec_len = registry.ro(owner).len();
        for ind in 0..registry_vec_len {
            let id = unsafe { registry.ro(&owner).get_unchecked(ind) };
            let socket = unsafe { self.sockets.get_unchecked_mut(*id) };
            match socket.state {
                SocketState::Idle => continue,
                SocketState::WriteRequest => {
                    socket.state = SocketState::Idle;
                    self.server.flush(owner, socket);
                }
                SocketState::CloseRequest => {
                    self.server.close(owner, socket);
                }
            }
        }
        registry.rw(owner).clear();
    }
}

pub fn entry_point<T>(server: T, addr: SocketAddr)
where
    T: SocketListener<Connection: Default>,
    [(); T::MAX_CONNECTIONS]:,
{
    LCellOwner::scope(|mut owner| {
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
            tick_machine.tick(|| selector.server.tick(&mut owner));
            for event in events.iter() {
                let token = event.token();
                if token == LISTENER_TOKEN {
                    selector.accept(&mut owner, &mut listener, &registry)
                } else {
                    let socket = unsafe { selector.sockets.get_unchecked_mut(token.0) };
                    selector.server.read(&mut owner, socket);
                }
            }
            selector.flush_registry(&mut owner, &registry)
        }
    })
}
