use std::{ops::DerefMut, time::Duration, usize};

use fast_collections::{Clear, Cursor, GetUnchecked, Push, Slab, Vec};

use crate::{tick_machine::TickMachine, Close, EntryPoint, Id, Open, Read, ReadError, Repo, Write};

pub struct MioTcpStream {
    stream: mio::net::TcpStream,
    token: mio::Token,
    is_closed: bool,
}

impl Read for MioTcpStream {
    type Ok = ();
    type Error = std::io::Error;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        read_buf.push_from_read(&mut self.stream)
    }
}

impl Write for MioTcpStream {
    type Error = std::io::Error;

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        write_buf.push_to_write(&mut self.stream)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Close for MioTcpStream {
    type Error = std::io::Error;

    type Registry = mio::Registry;

    fn close(&mut self, registry: &mut mio::Registry) -> Result<(), std::io::Error> {
        mio::event::Source::deregister(&mut self.stream, registry)
    }

    fn is_closed(&self) -> bool {
        self.is_closed
    }
}

impl Open for MioTcpStream {
    type Error = std::io::Error;

    type Registry = mio::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        mio::event::Source::register(
            &mut self.stream,
            registry,
            self.token,
            mio::Interest::READABLE,
        )
    }
}

pub struct SelectorPipe<T, S, const N: usize> {
    server: T,
    connections: Slab<Id<S>, S, N>,
    write_or_close_events: Vec<Id<S>, N>,
}

pub trait Selector<T> {
    fn tick(&mut self) -> Result<(), ()>;
    fn accept(&mut self, client: impl From<T>);
    fn read(&mut self, id: Id<T>) -> Result<(), ReadError>;
}

impl<
        T: Selector<SelectableChannel<S>>,
        S: Close<Registry = mio::Registry> + Write,
        const N: usize,
    > EntryPoint for SelectorPipe<T, SelectableChannel<S>, N>
where
    SelectableChannel<S>: From<MioTcpStream>,
    Id<SelectableChannel<S>>: From<usize>,
{
    fn entry_point(mut self, port: u16, tick: std::time::Duration) -> ! {
        let mut events = mio::Events::with_capacity(100);
        const LISTENER_INDEX: usize = usize::MAX;
        let mut poll = mio::Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        let listener = {
            let addr = format!("[::]:{port}").parse().unwrap();
            let mut listener = mio::net::TcpListener::bind(addr).unwrap();
            let lisetner_token = mio::Token(LISTENER_INDEX);
            let interest = mio::Interest::READABLE;
            mio::event::Source::register(&mut listener, &registry, lisetner_token, interest)
                .unwrap();
            listener
        };
        let mut tick_machine = TickMachine::new(tick);
        loop {
            poll.poll(&mut events, Some(Duration::ZERO)).unwrap();
            tick_machine.tick(|| self.server.tick().unwrap());
            for event in events.iter() {
                if event.token().0 == LISTENER_INDEX {
                    self.server
                        .accept(Into::<SelectableChannel<S>>::into(MioTcpStream {
                            stream: listener.accept().unwrap().0.into(),
                            token: mio::Token(0),
                            is_closed: false,
                        }))
                } else {
                    let socket_id = event.token();
                    match self.server.read(socket_id.0.into()) {
                        Ok(()) => {}
                        Err(err) => match err {
                            ReadError::NotFullRead => { /*TODO acc rate limit*/ }
                            ReadError::SocketClosed => {
                                self.request_socket_close(Id::from(socket_id.0))
                            }
                        },
                    };
                }
            }
            //self.flush_all_sockets(&mut registry);
        }
    }
}

pub struct SelectableChannel<T> {
    pub stream: T,
    state: SelectorState,
}

impl<T: Write> Write for SelectableChannel<T> {
    type Error = T::Error;

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
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

impl<'sel: 's, 's: 'cur, 'cur, T, S: Close + Write, const N: usize>
    SelectorPipe<T, SelectableChannel<S>, N>
{
    pub(crate) fn request_socket_close(&mut self, socket_id: Id<SelectableChannel<S>>) {
        let socket = unsafe { self.connections.get_unchecked_mut(&socket_id) };
        socket.state = SelectorState::CloseRequested;
        self.write_or_close_events
            .push(socket_id)
            .map_err(|_| ())
            .unwrap();
    }

    pub(crate) fn request_socket_flush(&mut self, socket_id: Id<SelectableChannel<S>>) {
        let socket = unsafe { self.connections.get_unchecked_mut(&socket_id) };
        if socket.state == SelectorState::Idle {
            socket.state = SelectorState::FlushRequested;
            self.write_or_close_events
                .push(socket_id)
                .map_err(|_| ())
                .unwrap();
        }
    }

    pub(self) fn flush_all_sockets<const LEN: usize>(
        &'sel mut self,
        registry: &'sel mut <S as Close>::Registry,
    ) where
        &'s mut S: Into<&'cur mut Cursor<u8, LEN>>,
        S: 's,
    {
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
                    if socket.flush().is_err() {
                        self.request_socket_close(socket_id);
                    }
                }
            }
        }
        self.write_or_close_events.clear();
    }
}
