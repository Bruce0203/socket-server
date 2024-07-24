use std::time::Duration;

use fast_collections::{Cursor, GetUnchecked, Push, Vec};
use packetize::{
    streaming_packets, Decode, Encode, ServerBoundPacketStream, SimplePacketStreamFormat,
};

use crate::{
    tick_machine::TickMachine, Close, Open, PollRead, ServerSocketChannel, ServerSocketService,
    SocketEvents, SocketId, Write,
};

pub struct MockStream {
    pub read_buffer: Cursor<u8, 1000>,
    pub write_buffer: Cursor<u8, 1000>,
}

impl MockStream {
    pub fn new() -> Self {
        Self {
            read_buffer: Default::default(),
            write_buffer: Default::default(),
        }
    }
}

impl Open for MockStream {
    type Error = ();
    type Registry = ();

    fn open(&mut self, id: usize, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl PollRead for MockStream {
    type Error = ();

    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        buffer.push_from_cursor(&mut self.write_buffer)?;
        Ok(())
    }
}

impl Write for MockStream {
    type Error = ();

    fn write<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        buffer.push_from_cursor(&mut self.read_buffer)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Close for MockStream {
    type Error = std::io::Error;
    type Registry = ();

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
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

#[derive(Default)]
pub struct MockChannel {
    connection_state: MockConnectionState,
}
