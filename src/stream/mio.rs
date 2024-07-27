use std::time::Duration;

use super::{Accept, Close, Flush, Id, Open, Read, ReadError, Write};
use crate::{
    connection::ConnectionPipe,
    selector::{SelectableChannel, Selector, SelectorListener},
    tick_machine::TickMachine,
};
use fast_collections::{AddWithIndex, Cursor};

use super::readable_byte_channel::PollRead;

pub struct MioTcpStream {
    stream: mio::net::TcpStream,
    pub token: mio::Token,
    pub is_closed: bool,
}

impl Read for MioTcpStream {
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        read_buf
            .push_from_read(&mut self.stream)
            .map_err(|_| ReadError::SocketClosed)?;
        Ok(())
    }
}

impl Accept<MioTcpStream> for MioTcpStream {
    fn accept(accept: MioTcpStream) -> Self {
        accept
    }

    fn get_stream(&mut self) -> &mut MioTcpStream {
        self
    }
}

impl Write for MioTcpStream {
    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        write_buf.push_to_write(&mut self.stream)
    }
}

impl Flush for MioTcpStream {
    type Error = std::io::Error;
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

impl<S, C, T: SelectorListener<S, C>, const MAX_CONNECTIONS: usize>
    Selector<T, S, C, MAX_CONNECTIONS>
{
    pub fn entry_point(&mut self, port: u16, tick_period: Duration) -> !
    where
        SelectableChannel<ConnectionPipe<S, C>>: Accept<MioTcpStream>,
        S: Close<Registry = mio::Registry> + Flush + PollRead + Open<Registry = mio::Registry>,
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
            tick_machine.tick(|| T::tick(self).unwrap());
            for event in events.iter() {
                if event.token().0 == LISTENER_INDEX {
                    if let Ok(socket_id) = self.connections.add_with_index(|index| {
                        let stream: mio::net::TcpStream = listener.accept().unwrap().0.into();
                        let token = mio::Token(*index);
                        Accept::accept(MioTcpStream {
                            stream,
                            token,
                            is_closed: false,
                        })
                    }) {
                        let socket_id = unsafe { Id::from(socket_id) };
                        let socket = self.get_mut(&socket_id);
                        socket.open(&mut registry).map_err(|_| ()).unwrap();
                        T::accept(self, socket_id)
                    }
                } else {
                    let socket_id = event.token();
                    self.handle_read(unsafe { Id::from(socket_id.0) }, &mut registry);
                }
            }
            self.flush_all_sockets(&mut registry)
        }
    }
}
