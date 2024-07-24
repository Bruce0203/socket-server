use std::marker::PhantomData;

use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

use crate::{socket_id::SocketId, ServerSocket, Socket, SocketRegistry};

#[derive(Default)]
pub struct ServerBoundPacket<T, F> {
    _marker: PhantomData<T>,
    f: F,
}

impl<T, F> ServerBoundPacket<T, F> {
    pub fn new(f: F) -> Self {
        Self {
            _marker: PhantomData,
            f,
        }
    }
}

impl<T: ClientBoundPacketStream> SocketRegistry<Socket<T>> {
    pub fn write_packet(
        &mut self,
        socket_id: &SocketId,
        packet: &T::BoundPacket,
    ) -> Result<(), ()> {
        let socket = self.get_socket_mut(socket_id);
        socket
            .stream
            .encode_client_bound_packet(packet, &mut socket.read_buffer)?;
        Ok(())
    }
}

impl<T: ServerBoundPacketStream, F> ServerSocket for ServerBoundPacket<T, F>
where
    F: FnMut(T::BoundPacket) -> Result<(), ()>,
{
    type Stream = T;
    type Registry = SocketRegistry<Socket<T>>;

    fn accept(&mut self, stream: Self::Stream, registry: &mut Self::Registry) {}

    fn poll_read(&mut self, socket_id: &SocketId, registry: &mut Self::Registry) {
        let socket = registry.get_socket_mut(socket_id);
        match socket
            .stream
            .decode_server_bound_packet(&mut socket.read_buffer)
        {
            Ok(packet) => {
                let f = &mut self.f;
                f(packet).unwrap();
            }
            Err(()) => registry.request_socket_close(socket_id),
        }
    }

    fn tick(&mut self, registry: &mut Self::Registry) -> Result<(), ()> {
        Ok(())
    }

    fn flush(&mut self, socket_id: &SocketId, registry: &mut Self::Registry) -> Result<(), ()> {
        Ok(())
    }

    fn flush_all_sockets(&mut self, registry: &mut Self::Registry) {}
}
