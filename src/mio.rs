use std::{io::Write, time::Duration};

use mio::event::Source;

use crate::{
    socket::{ServerSocketChannel, ServerSocketService, Socket, SocketEvents},
    tick_machine::TickMachine,
    SocketId,
};

pub fn entry_point<T: ServerSocketService>(service: T) -> !
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    let socket_server = &mut ServerSocketChannel::<mio::net::TcpStream, T>::new(service);
    let mut events = mio::Events::with_capacity(100);
    const LISTENER_INDEX: usize = usize::MAX;
    let mut poll = mio::Poll::new().unwrap();
    let mut mio_registry = poll.registry().try_clone().unwrap();
    let listener = {
        const PORT: u16 = 25525;
        let addr = format!("[::]:{PORT}").parse().unwrap();
        let mut listener = mio::net::TcpListener::bind(addr).unwrap();
        let lisetner_token = mio::Token(LISTENER_INDEX);
        let interest = mio::Interest::READABLE;
        mio::event::Source::register(&mut listener, &mio_registry, lisetner_token, interest)
            .unwrap();
        listener
    };
    const TICK: Duration = Duration::from_millis(50);
    let mut tick_machine = TickMachine::new(TICK);
    loop {
        poll.poll(&mut events, Some(Duration::ZERO)).unwrap();
        tick_machine.tick(|| socket_server.on_tick());
        let mut registry = SocketEvents::<T>::new();
        for event in events.iter() {
            if event.token().0 == LISTENER_INDEX {
                socket_server.on_accept(
                    listener.accept().unwrap().0,
                    &mut registry,
                    &mut mio_registry,
                )
            } else {
                socket_server.on_poll_read(event.token().0, &mut registry)
            }
        }
        socket_server.flush_toggled_write_or_close_events(&mut registry, &mut mio_registry);
    }
}

impl Socket for mio::net::TcpStream {
    type OpenError = std::io::Error;
    type ReadError = std::io::Error;
    type WriteError = std::io::Error;
    type CloseError = std::io::Error;
    type Registry = mio::Registry;

    fn open(
        &mut self,
        socket_id: &SocketId,
        registry: &mut mio::Registry,
    ) -> Result<(), Self::OpenError> {
        mio::Registry::register(
            &registry,
            self,
            mio::Token(socket_id.into()),
            mio::Interest::READABLE,
        )?;
        Ok(())
    }

    fn read<const N: usize>(
        &mut self,
        buffer: &mut fast_collections::Cursor<u8, N>,
    ) -> Result<(), std::io::Error> {
        buffer.push_from_read(self)?;
        Ok(())
    }

    fn write<const N: usize>(
        &mut self,
        buffer: &mut fast_collections::Cursor<u8, N>,
    ) -> Result<(), std::io::Error> {
        buffer.push_to_write(self)?;
        Ok(())
    }

    fn close(&mut self, registry: &mut mio::Registry) -> Result<(), std::io::Error> {
        self.flush()?;
        let _result = self.shutdown(std::net::Shutdown::Both);
        self.deregister(registry)?;
        Ok(())
    }
}
