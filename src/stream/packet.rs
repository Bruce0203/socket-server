use fast_collections::Cursor;
use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

use crate::{Accept, Close, Flush, Read, ReadError, Write};

pub struct ServerBoundPacketStreamPipe<T, S> {
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

impl<T: Read<Error = ReadError>, S: ClientBoundPacketStream + ServerBoundPacketStream> Read
    for ServerBoundPacketStreamPipe<T, S>
{
    type Ok = <S as ServerBoundPacketStream>::BoundPacket;
    type Error = ReadError;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<Self::Ok, T::Error> {
        self.stream.read(read_buf)?;
        Ok(self
            .state
            .decode_server_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
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

impl<T: Read<Error = ReadError>, S: ClientBoundPacketStream + ServerBoundPacketStream> Read
    for ClientBoundPacketStreamPipe<T, S>
{
    type Ok = <S as ClientBoundPacketStream>::BoundPacket;
    type Error = ReadError;
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<Self::Ok, T::Error> {
        self.stream.read(read_buf)?;
        Ok(self
            .state
            .decode_client_bound_packet(read_buf)
            .map_err(|()| ReadError::NotFullRead)?)
    }
}

impl<T: Write<Cursor<u8, LEN>>, const LEN: usize, S: ServerBoundPacketStream> Write<Cursor<u8, LEN>>
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
}

impl<T: Accept<A>, S: Default, A> Accept<A> for ClientBoundPacketStreamPipe<T, S> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            state: Default::default(),
        }
    }
}
