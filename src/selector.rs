use fast_collections::{Clear, Cursor, GetUnchecked, Push, Slab, Vec};

use super::stream::{Accept, Close, Flush, Id, Open, Read, ReadError, Write};
use crate::{
    connection::ConnectionPipe,
    stream::{packet::WritePacket, readable_byte_channel::PollRead},
};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Selector<T, S, C, const N: usize> {
    #[deref]
    #[deref_mut]
    server: T,
    pub(crate) connections: Slab<Id<C>, SelectableChannel<ConnectionPipe<S, C>>, N>,
    write_or_close_events: Vec<Id<C>, N>,
}

impl<T, S, C, const N: usize> Selector<T, S, C, N> {
    pub fn get(&self, id: &Id<C>) -> &SelectableChannel<ConnectionPipe<S, C>> {
        unsafe { self.connections.get_unchecked(id) }
    }

    pub fn get_mut(&mut self, id: &Id<C>) -> &mut SelectableChannel<ConnectionPipe<S, C>> {
        unsafe { self.connections.get_unchecked_mut(id) }
    }
}

impl<T, S, C, const N: usize> From<T> for Selector<T, S, C, N> {
    fn from(value: T) -> Self {
        Self {
            server: value,
            connections: Slab::new(),
            write_or_close_events: Vec::uninit(),
        }
    }
}

impl<T: Default, S, C, const N: usize> Default for Selector<T, S, C, N> {
    fn default() -> Self {
        Self {
            server: Default::default(),
            connections: Default::default(),
            write_or_close_events: Default::default(),
        }
    }
}

impl<T, S, C, const N: usize> Selector<T, S, C, N> {
    pub fn new(server: T) -> Self {
        Self {
            server,
            connections: Slab::new(),
            write_or_close_events: Vec::uninit(),
        }
    }
}

pub trait SelectorListener<T, C>: Sized {
    fn tick<const N: usize>(server: &mut Selector<Self, T, C, N>) -> Result<(), ()>;
    fn accept<const N: usize>(server: &mut Selector<Self, T, C, N>, id: Id<C>);
    fn read<const N: usize>(
        server: &mut Selector<Self, T, C, N>,
        id: Id<C>,
    ) -> Result<(), ReadError>;
    fn close<const N: usize>(server: &mut Selector<Self, T, C, N>, id: Id<C>);
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct SelectableChannel<T> {
    #[deref]
    #[deref_mut]
    pub stream: T,
    state: SelectorState,
}

impl<T: Accept<A>, A> Accept<A> for SelectableChannel<T> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            state: SelectorState::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T: Open> Open for SelectableChannel<T> {
    type Error = T::Error;
    type Registry = T::Registry;

    fn open(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}

impl<T: Close> Close for SelectableChannel<T> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Write> Write for SelectableChannel<T> {
    fn write<const N: usize>(&mut self, write: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.stream.write(write)
    }
}

impl<T: Flush> Flush for SelectableChannel<T> {
    type Error = T::Error;
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

impl<T: Read> Read for SelectableChannel<T> {
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        self.stream.read(read_buf)
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelectorState {
    #[default]
    Idle,
    CloseRequested,
    FlushRequested,
}

impl<T: SelectorListener<S, C>, C, S: Close + Flush + PollRead, const N: usize>
    Selector<T, S, C, N>
{
    pub fn request_socket_close(&mut self, id: Id<C>) {
        let socket = self.get_mut(&id);
        socket.state = SelectorState::CloseRequested;
        self.write_or_close_events.push(id).map_err(|_| ()).unwrap();
    }

    pub fn request_socket_flush(&mut self, id: Id<C>) {
        let socket = self.get_mut(&id);
        if socket.state == SelectorState::Idle {
            socket.state = SelectorState::FlushRequested;
            self.write_or_close_events.push(id).map_err(|_| ()).unwrap();
        }
    }

    pub fn handle_read(&mut self, id: Id<C>, _registry: &mut <S as Close>::Registry) {
        let mut f = || -> Result<(), ReadError> {
            let socket = self.get_mut(&id);
            socket.poll_read()?;
            T::read(self, id.clone())?;
            Ok(())
        };
        match f() {
            Ok(()) => {}
            Err(err) => match err {
                ReadError::NotFullRead => { /*TODO acc rate limit*/ }
                ReadError::SocketClosed => self.request_socket_close(id.clone()),
                ReadError::FlushRequest => self.request_socket_flush(id.clone()),
            },
        };
    }

    pub(crate) fn flush_all_sockets(&mut self, registry: &mut <S as Close>::Registry) {
        let len = self.write_or_close_events.len();
        let mut index = 0;
        while index < len {
            let socket_id = unsafe { self.write_or_close_events.get_unchecked(index) }.clone();
            let socket = unsafe { self.connections.get_unchecked_mut(&socket_id) };
            index += 1;
            match socket.state {
                SelectorState::Idle => continue,
                SelectorState::CloseRequested => {
                    T::close(self, socket_id.clone());
                    let socket = self.get_mut(&socket_id);
                    let _result = socket.stream.close(registry);
                }
                SelectorState::FlushRequested => {
                    socket.state = SelectorState::Idle;
                    if socket.stream.flush().is_err() {
                        self.request_socket_close(socket_id);
                    }
                }
            }
        }
        self.write_or_close_events.clear();
    }
}

impl<P, T: WritePacket<P>> WritePacket<P> for SelectableChannel<T> {
    fn send(&mut self, packet: P) -> Result<(), ReadError> {
        self.stream.send(packet)
    }
}

impl<T: PollRead> PollRead for SelectableChannel<T> {
    fn poll_read(&mut self) -> Result<(), ReadError> {
        self.stream.poll_read()
    }
}
