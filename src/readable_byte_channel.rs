use fast_collections::Cursor;

use crate::{Close, Open, Read, Write};

pub struct ReadableByteChannel<T, const LEN: usize> {
    pub stream: T,
    pub read_buf: Cursor<u8, LEN>,
}

impl<T, const LEN: usize> From<T> for ReadableByteChannel<T, LEN> {
    fn from(value: T) -> Self {
        Self {
            stream: value,
            read_buf: Cursor::new(),
        }
    }
}

impl<T: Read, const LEN: usize> Read for ReadableByteChannel<T, LEN> {
    type Ok = T::Ok;
    type Error = T::Error;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<T::Ok, Self::Error> {
        self.stream.read(read_buf)
    }
}

impl<T: Write, const LEN: usize> Write for ReadableByteChannel<T, LEN> {
    type Error = T::Error;

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

impl<T: Close, const LEN: usize> Close for ReadableByteChannel<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }
}

impl<T: Open, const LEN: usize> Open for ReadableByteChannel<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}

