use std::time::Duration;

use fast_collections::{AddWithIndex, Clear, Cursor, GetUnchecked, Push, Slab, Vec};

use crate::{
    stream::tcp::MioTcpStream, tick_machine::TickMachine, Accept, Close, Flush, Id, Open, Read,
    ReadError, Write,
};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Selector<T, S, const N: usize> {
    #[deref]
    #[deref_mut]
    server: T,
    pub(crate) connections: Slab<Id<S>, S, N>,
    write_or_close_events: Vec<Id<S>, N>,
}

impl<T, S, const N: usize> Selector<T, S, N> {
    pub fn get(&self, id: &Id<S>) -> &S {
        unsafe { self.connections.get_unchecked(id) }
    }

    pub fn get_mut(&mut self, id: &Id<S>) -> &mut S {
        unsafe { self.connections.get_unchecked_mut(id) }
    }
}

impl<T, S, const N: usize> From<T> for Selector<T, S, N> {
    fn from(value: T) -> Self {
        Self {
            server: value,
            connections: Slab::new(),
            write_or_close_events: Vec::uninit(),
        }
    }
}

impl<T: Default, S, const N: usize> Default for Selector<T, S, N> {
    fn default() -> Self {
        Self {
            server: Default::default(),
            connections: Default::default(),
            write_or_close_events: Default::default(),
        }
    }
}

impl<T, S, const N: usize> Selector<T, S, N> {
    pub fn new(server: T) -> Self {
        Self {
            server,
            connections: Slab::new(),
            write_or_close_events: Vec::uninit(),
        }
    }
}

pub trait SelectorListener<T>: Sized {
    fn tick<const N: usize>(server: &mut Selector<Self, T, N>) -> Result<(), ()>;
    fn accept<const N: usize>(server: &mut Selector<Self, T, N>, id: Id<T>);
    fn read<const N: usize>(server: &mut Selector<Self, T, N>, id: Id<T>) -> Result<(), ReadError>;
}

impl<S, T: SelectorListener<SelectableChannel<S>>, const MAX_CONNECTIONS: usize>
    Selector<T, SelectableChannel<S>, MAX_CONNECTIONS>
{
    pub fn entry_point<const LEN: usize>(mut self, port: u16, tick_period: Duration) -> !
    where
        SelectableChannel<S>: Accept<MioTcpStream>,
        S: Close<Registry = mio::Registry> + Write<Cursor<u8, LEN>> + Flush + Read + Open,
    {
        let mut events = mio::Events::with_capacity(MAX_CONNECTIONS);
        const LISTENER_INDEX: usize = usize::MAX;
        let mut poll = mio::Poll::new().unwrap();
        let mut registry = poll.registry().try_clone().unwrap();
        let listener = {
            let addr = format!("[::]:{port}").parse().unwrap();
            let mut listener = mio::net::TcpListener::bind(addr).unwrap();
            let lisetner_token = mio::Token(LISTENER_INDEX);
            let interest = mio::Interest::READABLE;
            mio::event::Source::register(&mut listener, &registry, lisetner_token, interest)
                .unwrap();
            listener
        };
        let mut tick_machine = TickMachine::new(tick_period);
        loop {
            poll.poll(&mut events, Some(Duration::ZERO)).unwrap();
            tick_machine.tick(|| T::tick(&mut self).unwrap());
            for event in events.iter() {
                if event.token().0 == LISTENER_INDEX {
                    if let Ok(socket_id) = self.connections.add_with_index(|index| {
                        let stream: mio::net::TcpStream = listener.accept().unwrap().0.into();
                        let token = mio::Token(*index);
                        SelectableChannel::accept(MioTcpStream {
                            stream,
                            token,
                            is_closed: false,
                        })
                    }) {
                        let socket_id = Id::from(socket_id);
                        let socket = self.get_mut(&socket_id);
                        socket.open(&mut registry).map_err(|_| ()).unwrap();
                        T::accept(&mut self, socket_id)
                    }
                } else {
                    let socket_id = event.token();

                    let socket_id = Id::from(socket_id.0);
                    match T::read(&mut self, socket_id.clone()) {
                        Ok(()) => {}
                        Err(err) => match err {
                            ReadError::NotFullRead => { /*TODO acc rate limit*/ }
                            ReadError::FlushRequest => self.request_socket_flush(socket_id),
                            ReadError::SocketClosed => self.request_socket_close(socket_id),
                        },
                    };
                }
            }
            self.flush_all_sockets(&mut registry)
        }
    }
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
}

impl<T: Open> Open for SelectableChannel<T> {
    type Error = T::Error;
    type Registry = T::Error;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
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

impl<T: Write<T2>, T2> Write<T2> for SelectableChannel<T> {
    fn write(&mut self, write: &mut T2) -> Result<(), Self::Error> {
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
    type Ok = T::Ok;

    type Error = T::Error;

    fn read<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<Self::Ok, Self::Error> {
        self.stream.read(read_buf)
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SelectorState {
    #[default]
    Idle,
    Closed,
    CloseRequested,
    FlushRequested,
}

impl<T, S: Close + Flush, const N: usize> Selector<T, SelectableChannel<S>, N> {
    pub fn request_socket_close(&mut self, id: Id<SelectableChannel<S>>) {
        let socket = self.get_mut(&id);
        socket.state = SelectorState::CloseRequested;
        self.write_or_close_events.push(id).map_err(|_| ()).unwrap();
    }

    pub fn request_socket_flush(&mut self, id: Id<SelectableChannel<S>>) {
        let socket = self.get_mut(&id);
        if socket.state == SelectorState::Idle {
            socket.state = SelectorState::FlushRequested;
            self.write_or_close_events.push(id).map_err(|_| ()).unwrap();
        }
    }

    pub(crate) fn flush_all_sockets(&mut self, registry: &mut <S as Close>::Registry) {
        let len = self.write_or_close_events.len();
        let mut index = 0;
        while index < len {
            let socket_id = unsafe { self.write_or_close_events.get_unchecked(index) }.clone();
            let socket = unsafe { self.connections.get_unchecked_mut(&socket_id) };
            index += 1;
            match socket.state {
                SelectorState::Idle | SelectorState::Closed => continue,
                SelectorState::CloseRequested => {
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
