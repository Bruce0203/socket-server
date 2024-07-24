use fast_collections::Cursor;
use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

use crate::{Read, ReadError, Write};

pub struct ServerPacketStreamPipe<T, S> {
    pub socket: T,
    state: S,
}

impl<T, S: Default> From<T> for ServerPacketStreamPipe<T, S> {
    fn from(value: T) -> Self {
        Self {
            socket: value,
            state: S::default(),
        }
    }
}

impl<T, S> ServerPacketStreamPipe<T, S> {
    pub fn new(value: T, state: S) -> Self {
        Self {
            socket: value,
            state,
        }
    }
}

impl<T: Read<Error = ReadError>, S: ClientBoundPacketStream + ServerBoundPacketStream> Read
    for ServerPacketStreamPipe<T, S>
{
    type Ok = <S as ServerBoundPacketStream>::BoundPacket;
    type Error = ReadError;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<Self::Ok, T::Error> {
        self.socket.read(read_buf)?;
        Ok(self
            .state
            .decode_server_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
    }
}

pub struct ClientPacketStreamPipe<T, S> {
    socket: T,
    state: S,
}

impl<T, S> ClientPacketStreamPipe<T, S> {
    pub fn new(value: T, state: S) -> Self {
        Self {
            socket: value,
            state,
        }
    }
}

impl<T: Read<Error = ReadError>, S: ClientBoundPacketStream + ServerBoundPacketStream> Read
    for ClientPacketStreamPipe<T, S>
{
    type Ok = <S as ClientBoundPacketStream>::BoundPacket;
    type Error = ReadError;
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<Self::Ok, T::Error> {
        self.socket.read(read_buf)?;
        Ok(self
            .state
            .decode_client_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
    }
}

impl<T: Write, S> Write for ClientPacketStreamPipe<T, S> {
    type Error = T::Error;

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.socket.write(write_buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.socket.flush()
    }
}

impl<T: Write, S> Write for ServerPacketStreamPipe<T, S> {
    type Error = T::Error;

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.socket.write(write_buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.socket.flush()
    }
}

