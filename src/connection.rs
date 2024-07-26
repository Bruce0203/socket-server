use fast_collections::Cursor;

use crate::{
    stream::{
        packet::WritePacket,
        readable_byte_channel::{PollRead, ReceivePacket},
    },
    Accept, Close, Flush, Open, Read, ReadError, Write,
};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct ConnectionPipe<T, S> {
    pub stream: T,
    #[deref]
    #[deref_mut]
    connection: S,
}

impl<T: Accept<A>, S: Default, A> Accept<A> for ConnectionPipe<T, S> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            connection: S::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T, S: Default> From<T> for ConnectionPipe<T, S> {
    fn from(value: T) -> Self {
        Self {
            stream: value,
            connection: S::default(),
        }
    }
}
impl<T: Open, S> Open for ConnectionPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}

impl<T: Close, S> Close for ConnectionPipe<T, S> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Write<T2>, S, T2> Write<T2> for ConnectionPipe<T, S> {
    fn write(&mut self, write: &mut T2) -> Result<(), Self::Error> {
        self.stream.write(write)
    }
}

impl<T: Flush, S> Flush for ConnectionPipe<T, S> {
    type Error = T::Error;

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

impl<T: Read, S> Read for ConnectionPipe<T, S> {
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        self.stream.read(read_buf)
    }
}

impl<P, T: WritePacket<P>, S> WritePacket<P> for ConnectionPipe<T, S> {
    fn send(&mut self, packet: P) -> Result<(), crate::ReadError> {
        self.stream.send(packet)
    }
}

impl<T: PollRead, S> PollRead for ConnectionPipe<T, S> {
    fn poll_read(&mut self) -> Result<(), ReadError> {
        self.stream.poll_read()
    }
}

impl<T: ReceivePacket<P>, S, P> ReceivePacket<P> for ConnectionPipe<T, S> {
    fn recv(&mut self) -> Result<P, crate::ReadError> {
        self.stream.recv()
    }
}
