use fast_collections::Cursor;

use crate::{Accept, Close, Flush, Open, Read, ReadError, Write};

pub struct WritableByteChannel<T, const LEN: usize> {
    pub stream: T,
    pub write_buf: Cursor<u8, LEN>,
}

impl<T, const LEN: usize> From<T> for WritableByteChannel<T, LEN> {
    fn from(value: T) -> Self {
        Self {
            stream: value,
            write_buf: Cursor::new(),
        }
    }
}

impl<T: Accept<A>, A, const LEN: usize> Accept<A> for WritableByteChannel<T, LEN> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            write_buf: Cursor::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T: Read, const LEN: usize> Read for WritableByteChannel<T, LEN> {
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        self.stream.read(read_buf)
    }
}

impl<T: Write<Cursor<u8, LEN>>, const LEN: usize> Write<Cursor<u8, LEN>>
    for WritableByteChannel<T, LEN>
{
    fn write(&mut self, write_buf: &mut Cursor<u8, LEN>) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }
}

impl<T: Flush + Write<Cursor<u8, LEN>>, const LEN: usize> Flush for WritableByteChannel<T, LEN> {
    type Error = <T as Flush>::Error;
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.write(&mut self.write_buf)?;
        self.stream.flush()
    }
}

impl<T: Close, const LEN: usize> Close for WritableByteChannel<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }
}

impl<T: Open, const LEN: usize> Open for WritableByteChannel<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}
