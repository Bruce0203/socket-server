use std::ops::DerefMut;

use fast_collections::Cursor;
use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

use crate::{Accept, Close, Flush, Open, Read, ReadError, Write};

use super::writable_byte_channel::WritableByteChannel;

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct ServerBoundPacketStreamPipe<T, S> {
    #[deref]
    #[deref_mut]
    pub stream: T,
    state: S,
}

impl<T, S: Default> From<T> for ServerBoundPacketStreamPipe<T, S> {
    fn from(value: T) -> Self {
        Self {
            stream: value,
            state: S::default(),
        }
    }
}

impl<T, S> ServerBoundPacketStreamPipe<T, S> {
    pub fn new(value: T, state: S) -> Self {
        Self {
            stream: value,
            state,
        }
    }
}

impl<T: Read<(), Error = ReadError>, S: ServerBoundPacketStream> Read<()>
    for ServerBoundPacketStreamPipe<T, S>
{
    type Error = ReadError;

    fn read<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<(), <ServerBoundPacketStreamPipe<T, S> as Read<()>>::Error> {
        self.stream.read(read_buf)
    }
}

pub struct ClientBoundPacketStreamPipe<T, S> {
    stream: T,
    state: S,
}

impl<T, S> ClientBoundPacketStreamPipe<T, S> {
    pub fn new(value: T, state: S) -> Self {
        Self {
            stream: value,
            state,
        }
    }
}

pub trait ReadPacket<T> {
    fn recv<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<T, ReadError>;
}

impl<T, S: ClientBoundPacketStream> ReadPacket<S::BoundPacket>
    for ClientBoundPacketStreamPipe<T, S>
{
    fn recv<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<S::BoundPacket, ReadError> {
        Ok(self
            .state
            .decode_client_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
    }
}

impl<T, S: ServerBoundPacketStream> ReadPacket<S::BoundPacket>
    for ServerBoundPacketStreamPipe<T, S>
{
    fn recv<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<S::BoundPacket, ReadError> {
        Ok(self
            .state
            .decode_server_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
    }
}

impl<T: Read<T2, Error = ReadError>, T2, S> Read<T2> for ClientBoundPacketStreamPipe<T, S> {
    type Error = ReadError;
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<T2, Self::Error> {
        self.stream.read(read_buf)
    }
}

impl<T: Write<Cursor<u8, LEN>>, const LEN: usize, S> Write<Cursor<u8, LEN>>
    for ClientBoundPacketStreamPipe<T, S>
{
    fn write(&mut self, write_buf: &mut Cursor<u8, LEN>) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }
}

impl<T: Flush, S> Flush for ClientBoundPacketStreamPipe<T, S> {
    type Error = T::Error;

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

impl<T: Write<T2>, T2, S> Write<T2> for ServerBoundPacketStreamPipe<T, S> {
    fn write(&mut self, write_buf: &mut T2) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }
}

impl<T: Flush, S> Flush for ServerBoundPacketStreamPipe<T, S> {
    type Error = T::Error;
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

pub trait WritePacket<T> {
    fn send(&mut self, packet: T) -> Result<(), ReadError>;
}

impl<T, S: ClientBoundPacketStream, const LEN: usize>
    ServerBoundPacketStreamPipe<WritableByteChannel<T, LEN>, S>
{
    pub fn write(&mut self, packet: S::BoundPacket) -> Result<(), ReadError> {
        self.state
            .encode_client_bound_packet(&packet, &mut self.stream.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<T, S: ClientBoundPacketStream, const LEN: usize> WritePacket<S::BoundPacket>
    for ServerBoundPacketStreamPipe<WritableByteChannel<T, LEN>, S>
{
    fn send(&mut self, packet: S::BoundPacket) -> Result<(), ReadError> {
        self.state
            .encode_client_bound_packet(&packet, &mut self.stream.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<
        T: DerefMut<Target = WritableByteChannel<T2, LEN>>,
        T2,
        S: ClientBoundPacketStream,
        const LEN: usize,
    > WritePacket<S::BoundPacket> for ServerBoundPacketStreamPipe<T, S>
{
    fn send(&mut self, packet: S::BoundPacket) -> Result<(), ReadError> {
        self.state
            .encode_client_bound_packet(&packet, &mut self.stream.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<T, S: ServerBoundPacketStream, const LEN: usize> WritePacket<S::BoundPacket>
    for ClientBoundPacketStreamPipe<WritableByteChannel<T, LEN>, S>
{
    fn send(&mut self, packet: S::BoundPacket) -> Result<(), ReadError> {
        self.state
            .encode_server_bound_packet(&packet, &mut self.stream.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<
        T: DerefMut<Target = WritableByteChannel<T2, LEN>>,
        T2,
        S: ServerBoundPacketStream,
        const LEN: usize,
    > WritePacket<S::BoundPacket> for ClientBoundPacketStreamPipe<T, S>
{
    fn send(&mut self, packet: S::BoundPacket) -> Result<(), ReadError> {
        self.state
            .encode_server_bound_packet(&packet, &mut self.stream.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<T: Close, S> Close for ServerBoundPacketStreamPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Close, S> Close for ClientBoundPacketStreamPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Accept<A>, S: Default, A> Accept<A> for ServerBoundPacketStreamPipe<T, S> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            state: Default::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T: Accept<A>, S: Default, A> Accept<A> for ClientBoundPacketStreamPipe<T, S> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            state: Default::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T: Open, S> Open for ClientBoundPacketStreamPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}

impl<T: Open, S> Open for ServerBoundPacketStreamPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}
