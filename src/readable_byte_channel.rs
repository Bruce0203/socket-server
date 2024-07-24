use fast_collections::Cursor;

use crate::{Accept, Close, Flush, Open, Read, Write};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct ReadableByteChannel<T, const LEN: usize> {
    #[deref]
    #[deref_mut]
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

impl<T: Accept<A>, const LEN: usize, A> Accept<A> for ReadableByteChannel<T, LEN> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            read_buf: Cursor::default(),
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

impl<T: Write<T2>, T2, const LEN: usize> Write<T2> for ReadableByteChannel<T, LEN> {
    fn write(&mut self, write_buf: &mut T2) -> Result<(), Self::Error> {
        self.stream.write(write_buf)
    }
}

impl<T: Flush, const LEN: usize> Flush for ReadableByteChannel<T, LEN> {
    type Error = T::Error;
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
