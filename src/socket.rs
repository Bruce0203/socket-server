use std::marker::PhantomData;

use fast_collections::{
    AddWithIndex, Clear, Cursor, GetUnchecked, Push, RemoveUnchecked, Slab, Vec,
};
use nonmax::NonMaxUsize;
use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

pub struct SocketId(pub(self) NonMaxUsize);

impl SocketId {
    pub(self) fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    pub(self) fn from(value: usize) -> Self {
        Self(unsafe { NonMaxUsize::new_unchecked(value) })
    }
}

impl Into<usize> for &SocketId {
    fn into(self) -> usize {
        self.0.get()
    }
}
pub trait ServerSocketService: Sized {
    const MAX_CONNECTIONS: usize;
    const WRITE_BUFFER_LENGTH: usize;
    const READ_BUFFER_LENGTH: usize;
    type ConnectionState: ServerBoundPacketStream + ClientBoundPacketStream + Default;
    type Channel: Channel;

    fn tick(&mut self);

    fn read(
        &mut self,
        socket_id: &SocketId,
        registry: &mut SocketEvents<Self>,
        packet: <Self::ConnectionState as ServerBoundPacketStream>::BoundPacket,
    ) -> Result<(), ()>
    where
        [(); Self::MAX_CONNECTIONS]:,
        [(); Self::WRITE_BUFFER_LENGTH]:,
        [(); Self::READ_BUFFER_LENGTH]:;

    fn accept(&mut self, socket_id: &SocketId, registry: &mut SocketEvents<Self>)
    where
        [(); Self::MAX_CONNECTIONS]:,
        [(); Self::WRITE_BUFFER_LENGTH]:,
        [(); Self::READ_BUFFER_LENGTH]:;

    fn close(&mut self, socket_id: &SocketId, registry: &mut SocketEvents<Self>)
    where
        [(); Self::MAX_CONNECTIONS]:,
        [(); Self::WRITE_BUFFER_LENGTH]:,
        [(); Self::READ_BUFFER_LENGTH]:;
}

pub enum SocketReadError {
    NotFullRead,
    SocketClosed,
}

pub trait Channel: Default {
    fn on_read<T: ServerSocketService>(
        &mut self,
        socket_id: &SocketId,
        registry: &mut SocketEvents<T>,
    ) -> Result<(), SocketReadError>
    where
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:;

    fn on_write<T: ServerSocketService>(
        &mut self,
        socket_id: &SocketId,
        registry: &mut SocketEvents<T>,
    ) -> Result<(), ()>
    where
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:;
}

pub struct ServerSocketChannel<Stream, T: ServerSocketService>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    service: T,
    connections: Slab<SocketId, Connection<Stream, T>, { T::MAX_CONNECTIONS }>,
}

pub struct Connection<Stream, T: ServerSocketService>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    stream: Stream,
    channel: T::Channel,
}

impl<Stream, T: ServerSocketService> Connection<Stream, T>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            channel: Default::default(),
        }
    }
}

impl<Stream, T: ServerSocketService> ServerSocketChannel<Stream, T>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    pub fn new(listener: T) -> Self {
        Self {
            connections: Slab::new(),
            service: listener,
        }
    }
}

pub struct SocketChannel<T: ServerSocketService>
where
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::MAX_CONNECTIONS]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    connection_state: T::ConnectionState,
    is_write_or_close_event_toggled: bool,
    is_closed: bool,
    pub read_buffer: Cursor<u8, { T::READ_BUFFER_LENGTH }>,
    pub write_buffer: Cursor<u8, { T::WRITE_BUFFER_LENGTH }>,
    _marker: PhantomData<T>,
}

pub trait Socket {
    type OpenError;
    type ReadError;
    type WriteError;
    type CloseError;
    type Registry;

    fn open(
        &mut self,
        socket_id: &SocketId,
        registry: &mut Self::Registry,
    ) -> Result<(), Self::OpenError>;

    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::ReadError>;

    fn write<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>)
        -> Result<(), Self::WriteError>;

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::CloseError>;
}

pub struct SocketEvents<T: ServerSocketService>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    pub(crate) write_or_close_events: Vec<SocketId, { T::MAX_CONNECTIONS }>,
    pub(crate) connections: Slab<SocketId, SocketChannel<T>, { T::MAX_CONNECTIONS }>,
}

impl<T: ServerSocketService> SocketEvents<T>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    pub fn new() -> Self {
        Self {
            write_or_close_events: Vec::uninit(),
            connections: Slab::new(),
        }
    }
    pub fn write_packet(
        &mut self,
        socket_id: &SocketId,
        packet: &<T::ConnectionState as ClientBoundPacketStream>::BoundPacket,
    ) -> Result<(), ()> {
        let socket = unsafe { self.connections.get_unchecked_mut(socket_id) };
        socket.is_write_or_close_event_toggled = true;
        socket
            .connection_state
            .encode_client_bound_packet(packet, &mut socket.write_buffer)?;
        self.write_or_close_events
            .push(socket_id.clone())
            .map_err(|_| ())?;
        Ok(())
    }

    pub fn close_socket(&mut self, socket_id: &SocketId) {
        let socket = unsafe { self.connections.get_unchecked_mut(socket_id.into()) };
        if !socket.is_closed && !socket.is_write_or_close_event_toggled {
            socket.is_closed = true;
            socket.is_write_or_close_event_toggled = true;
            unsafe { self.write_or_close_events.push_unchecked(socket_id.clone()) }
        }
    }

    pub(crate) fn register_write_event(&mut self, socket_id: &SocketId) {
        let socket = unsafe { self.connections.get_unchecked(socket_id) };
        if !socket.is_write_or_close_event_toggled {
            self.write_or_close_events
                .push(socket_id.clone())
                .map_err(|_| ())
                .unwrap();
        }
    }
}

impl<Stream: Socket, T: ServerSocketService> ServerSocketChannel<Stream, T>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    pub fn on_accept(
        &mut self,
        stream: Stream,
        registry: &mut SocketEvents<T>,
        socket_registry: &mut <Stream as Socket>::Registry,
    ) {
        if let Ok(socket_id) = self
            .connections
            .add_with_index(|_index| Connection::new(stream))
        {
            let socket_id = SocketId::from(socket_id);
            let socket = unsafe { self.connections.get_unchecked_mut(&socket_id) };
            registry
                .connections
                .add_with_index(|_index| SocketChannel::<T> {
                    read_buffer: Default::default(),
                    write_buffer: Default::default(),
                    connection_state: Default::default(),
                    is_write_or_close_event_toggled: Default::default(),
                    is_closed: Default::default(),
                    _marker: PhantomData,
                })
                .unwrap();

            socket
                .stream
                .open(&socket_id, socket_registry)
                .map_err(|_| ())
                .unwrap();
            self.service.accept(&socket_id, registry);
        } else {
        }
    }

    pub fn on_tick(&mut self) {
        self.service.tick();
    }

    pub fn on_poll_read(&mut self, token: usize, registry: &mut SocketEvents<T>)
    where
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:,
    {
        let socket_id = SocketId::from(token);
        let mut read = || -> Result<(), ()> {
            let stream = unsafe { self.connections.get_unchecked_mut(&socket_id) };
            let socket = unsafe { registry.connections.get_unchecked_mut(&socket_id) };
            stream
                .stream
                .read(&mut socket.read_buffer)
                .map_err(|_| ())?;
            if let Err(err) = stream.channel.on_read(&socket_id, registry) {
                return match err {
                    SocketReadError::NotFullRead => Ok(()),
                    SocketReadError::SocketClosed => Err(()),
                };
            }
            while {
                let socket = unsafe { registry.connections.get_unchecked_mut(&socket_id) };
                socket.read_buffer.remaining().clone()
            } != 0
            {
                let socket = unsafe { registry.connections.get_unchecked_mut(&socket_id) };
                let packet = socket
                    .connection_state
                    .decode_server_bound_packet(&mut socket.read_buffer)
                    .map_err(|_| ())?;
                self.service.read(&socket_id, registry, packet)?;
            }
            let socket = unsafe { registry.connections.get_unchecked_mut(&socket_id) };
            socket.read_buffer.clear();
            Ok(())
        };
        if let Err(()) = read() {
            registry.close_socket(&socket_id)
        }
    }

    pub fn flush_toggled_write_or_close_events(
        &mut self,
        registry: &mut SocketEvents<T>,
        socket_registry: &mut <Stream as Socket>::Registry,
    ) {
        let mut index = 0;
        while index < registry.write_or_close_events.len() {
            let socket_id = unsafe { registry.write_or_close_events.get_unchecked_mut(index) };
            let socket_id = &socket_id.clone();
            index += 1;
            let stream = unsafe { self.connections.get_unchecked_mut(socket_id) };
            let socket = unsafe { registry.connections.get_unchecked_mut(socket_id) };
            let is_closed = socket.is_closed;
            socket.is_write_or_close_event_toggled = false;

            let mut f = || -> Result<(), ()> {
                stream.channel.on_write(socket_id, registry)?;
                let socket = unsafe { registry.connections.get_unchecked_mut(socket_id) };
                stream
                    .stream
                    .write(&mut socket.write_buffer)
                    .map_err(|_| ())?;
                Ok(())
            };
            if is_closed || f().is_err() {
                let _result = stream.stream.close(socket_registry);
                self.service.close(socket_id, registry);
                unsafe { self.connections.remove_unchecked(socket_id) };
                unsafe { registry.connections.remove_unchecked(socket_id) };
            }
        }
    }
}

#[cfg(test)]
mod test {
    use fast_collections::Pop;
    use packetize::Encode;

    use crate::{
        mock::{MockHandShakeC2s, MockListener, MockStream, ServerBoundPacket},
        socket::{ServerSocketChannel, SocketEvents},
        SocketId,
    };

    #[test]
    fn test_server_socket() {
        type App = MockListener;
        let mut server = ServerSocketChannel::<MockStream<App>, _>::new(App::default());
        {
            let mut registry = SocketEvents::<App>::new();
            let mut stream = MockStream::new();
            MockHandShakeC2s {
                protocol_version: 123,
            }
            .encode(&mut stream.write_buffer)
            .unwrap();
            server.on_accept(stream, &mut registry, &mut ());
            server.on_poll_read(0, &mut registry);
            let received_packet = match server.service.received_packets.pop().unwrap() {
                ServerBoundPacket::MockHandShakeC2s(value) => value,
            };
            assert_eq!(received_packet.protocol_version, 123);
            registry.close_socket(&SocketId::from(0));
            server.on_tick();
            server.flush_toggled_write_or_close_events(&mut registry, &mut ());
            assert_eq!(server.service.is_closed, true);
        }
    }
}
