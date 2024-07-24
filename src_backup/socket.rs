use fast_collections::{AddWithIndex, Clear, Cursor, GetUnchecked, Push, Slab, Vec};
use fast_delegate::delegate;

use crate::socket_id::SocketId;

pub trait ServerSocket {
    type Stream;
    type Registry;
    fn accept(&mut self, stream: Self::Stream, registry: &mut Self::Registry);
    fn poll_read(&mut self, socket_id: &SocketId, registry: &mut Self::Registry);
    fn tick(&mut self, registry: &mut Self::Registry) -> Result<(), ()>;
    fn flush(&mut self, socket_id: &SocketId, registry: &mut Self::Registry) -> Result<(), ()>;
    fn flush_all_sockets(&mut self, registry: &mut Self::Registry);
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SocketState {
    #[default]
    Idle,
    Closed,
    CloseRequested,
    FlushRequested,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Socket<T> {
    #[deref]
    #[deref_mut]
    pub(crate) stream: T,
    state: SocketState,
    pub(crate) read_buffer: Cursor<u8, 1000>,
    pub(crate) write_buffer: Cursor<u8, 1000>,
}

pub trait Stream {
    type Registry;
    type Error;
    fn open(&mut self, token: usize, registry: &mut Self::Registry) -> Result<(), Self::Error>;
    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error>;
}

pub trait Write {
    type Error;
    fn write<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub trait Read {
    type Error;
    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error>;
}

#[delegate]
pub trait Tick {
    fn tick(&mut self) -> Result<(), ()>;
}

pub struct SocketServer<S, T> {
    server: S,
    registry: SocketRegistry<T>,
}

impl<S, T> SocketServer<S, T> {
    pub fn new(server: S) -> Self {
        Self {
            server,
            registry: SocketRegistry::new(),
        }
    }
}

#[derive(Default)]
pub struct SocketRegistry<T> {
    sockets: Slab<SocketId, T, 100>,
    write_or_close_events: Vec<SocketId, 100>,
}

impl<T> SocketRegistry<T> {
    pub fn new() -> Self {
        Self {
            sockets: Slab::new(),
            write_or_close_events: Vec::uninit(),
        }
    }
}

impl<T> SocketRegistry<T> {
    pub fn get_socket_mut(&mut self, socket_id: &SocketId) -> &mut T {
        unsafe { self.sockets.get_unchecked_mut(socket_id) }
    }
}

impl<T> SocketRegistry<Socket<T>> {
    pub(crate) fn request_socket_close(&mut self, socket_id: &SocketId) {
        let socket = self.get_socket_mut(socket_id);
        socket.state = SocketState::CloseRequested;
        self.write_or_close_events
            .push(socket_id.clone())
            .map_err(|_| ())
            .unwrap();
    }

    pub(crate) fn request_socket_flush(&mut self, socket_id: &SocketId) {
        let socket = self.get_socket_mut(socket_id);
        if socket.state == SocketState::Idle {
            socket.state = SocketState::FlushRequested;
            self.write_or_close_events
                .push(socket_id.clone())
                .map_err(|_| ())
                .unwrap();
        }
    }
}

impl<S, T: Stream + Write + Read> ServerSocket for SocketServer<S, Socket<T>>
where
    S: ServerSocket<Stream: Default, Registry = SocketRegistry<Socket<T>>>,
{
    type Stream = T;
    type Registry = T::Registry;

    fn accept(&mut self, stream: Self::Stream, registry: &mut T::Registry) {
        if let Ok(socket_id) = self.registry.sockets.add_with_index(|_index| Socket {
            stream,
            read_buffer: Cursor::new(),
            write_buffer: Cursor::new(),
            state: Default::default(),
        }) {
            let socket_id = &SocketId::from(socket_id);
            let socket = self.registry.get_socket_mut(socket_id);
            match socket.stream.open(socket_id.into(), registry) {
                Ok(_) => self.server.accept(Default::default(), &mut self.registry),
                Err(_) => {
                    let _result = socket.stream.close(registry);
                }
            };
        };
    }

    fn poll_read(&mut self, socket_id: &SocketId, registry: &mut T::Registry) {
        let socket = self.registry.get_socket_mut(socket_id);
        match socket.stream.read(&mut socket.read_buffer) {
            Ok(_) => self.server.poll_read(socket_id, &mut self.registry),
            Err(_) => {
                let _result = socket.stream.close(registry);
            }
        }
    }

    fn tick(&mut self, _registry: &mut T::Registry) -> Result<(), ()> {
        self.server.tick(&mut self.registry)
    }

    fn flush(&mut self, socket_id: &SocketId, _registry: &mut T::Registry) -> Result<(), ()> {
        self.server.flush(socket_id, &mut self.registry)
    }

    fn flush_all_sockets(&mut self, registry: &mut Self::Registry) {
        let len = self.registry.write_or_close_events.len();
        let mut index = 0;
        while index < len {
            let socket_id =
                &unsafe { self.registry.write_or_close_events.get_unchecked(index) }.clone();
            let socket = self.registry.get_socket_mut(socket_id);
            index += 1;
            match socket.state {
                SocketState::Idle | SocketState::Closed => continue,
                SocketState::CloseRequested => {
                    let _result = socket.stream.close(registry);
                }
                SocketState::FlushRequested => {
                    socket.state = SocketState::Idle;
                    let mut f = || -> Result<(), ()> {
                        self.server.flush(socket_id, &mut self.registry)?;
                        let socket = self.registry.get_socket_mut(socket_id);
                        socket.stream.flush().map_err(|_| ())?;
                        socket
                            .stream
                            .write(&mut socket.write_buffer)
                            .map_err(|_| ())?;
                        Ok(())
                    };
                    if f().is_err() {
                        self.registry.request_socket_close(socket_id);
                    }
                }
            }
        }
        self.registry.write_or_close_events.clear();
    }
}
