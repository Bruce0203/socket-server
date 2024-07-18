use fast_collections::{Cursor, Push, Vec};
use packetize::{
    streaming_packets, Decode, Encode, ServerBoundPacketStream, SimplePacketStreamFormat,
};

use crate::{ServerSocketService, Socket, SocketEvents, SocketId};

pub struct MockStream<T: ServerSocketService>
where
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    pub read_buffer: Cursor<u8, { T::WRITE_BUFFER_LENGTH }>,
    pub write_buffer: Cursor<u8, { T::READ_BUFFER_LENGTH }>,
}

impl<T: ServerSocketService> MockStream<T>
where
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    pub fn new() -> Self {
        Self {
            read_buffer: Default::default(),
            write_buffer: Default::default(),
        }
    }
}
impl<T: ServerSocketService> Socket for MockStream<T>
where
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    type OpenError = ();
    type ReadError = ();
    type WriteError = ();
    type CloseError = ();
    type Registry = ();

    fn open(
        &mut self,
        _socket_id: &SocketId,
        _registry: &mut Self::Registry,
    ) -> Result<(), Self::OpenError> {
        Ok(())
    }

    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::ReadError> {
        buffer.push_from_cursor(&mut self.write_buffer)?;
        Ok(())
    }

    fn write<const N: usize>(
        &mut self,
        buffer: &mut Cursor<u8, N>,
    ) -> Result<(), Self::WriteError> {
        buffer.push_from_cursor(&mut self.read_buffer)?;
        Ok(())
    }

    fn close(&mut self, _registry: &mut Self::Registry) -> Result<(), Self::CloseError> {
        let socket_closed = std::any::type_name::<Self>();
        dbg!(socket_closed);
        Ok(())
    }
}

#[streaming_packets(SimplePacketStreamFormat)]
#[derive(Default)]
pub enum MockConnectionState {
    #[default]
    HandShake(MockHandShakeC2s, MockHandShakeS2c),
}

#[derive(Encode, Decode, Debug)]
pub struct MockHandShakeC2s {
    pub protocol_version: u32,
}

#[derive(Encode, Decode, Debug)]
pub struct MockHandShakeS2c {
    protocol_version: u32,
}

#[derive(Default)]
pub struct MockListener {
    pub received_packets: Vec<ServerBoundPacket, 100>,
    pub is_closed: bool,
}
impl ServerSocketService for MockListener {
    const MAX_CONNECTIONS: usize = 3;
    const WRITE_BUFFER_LENGTH: usize = 4096;
    const READ_BUFFER_LENGTH: usize = 4096;
    type ConnectionState = MockConnectionState;

    fn tick(&mut self) {}
    fn read(
        &mut self,
        _socket_id: &SocketId,
        _registry: &mut SocketEvents<Self>,
        packet: <Self::ConnectionState as ServerBoundPacketStream>::BoundPacket,
    ) -> Result<(), ()> {
        let result = self.received_packets.push(packet);
        Ok(())
    }
    fn accept(&mut self, _socket_id: &SocketId, _registry: &mut SocketEvents<Self>) {}
    fn close(&mut self, _socket_id: &SocketId, _registry: &mut SocketEvents<Self>) {
        self.is_closed = true;
    }
}
