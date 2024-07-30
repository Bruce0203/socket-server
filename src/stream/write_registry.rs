use fast_collections::{Cursor, Vec};

use super::{Accept, Close, Flush, Open, Read, Write};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct SelectorWriteRegistry<T, const N: usize> {
    #[deref]
    #[deref_mut]
    stream: T,
    vec: Vec<usize, N>,
}

pub trait RegisterWrite<T, const N: usize> {
    fn get_selector_registry_mut(&mut self) -> &mut SelectorWriteRegistry<T, N>;
}

impl<T, const N: usize> SelectorWriteRegistry<T, N> {}

impl<T: Accept<P>, const N: usize, P> Accept<P> for SelectorWriteRegistry<T, N> {
    fn get_stream(&mut self) -> &mut P {
        self.stream.get_stream()
    }

    fn accept(accept: P) -> Self {
        SelectorWriteRegistry {
            stream: T::accept(accept),
            vec: Vec::uninit(),
        }
    }
}

impl<T: Read, const LEN: usize> Read for SelectorWriteRegistry<T, LEN> {
    fn read<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<(), super::ReadError> {
        self.stream.read(read_buf)
    }
}

impl<T: Write, const LEN: usize> Write for SelectorWriteRegistry<T, LEN> {
    fn write<const N: usize>(&mut self, write: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.stream.write(write)
    }
}

impl<T: Flush, const LEN: usize> Flush for SelectorWriteRegistry<T, LEN> {
    type Error = T::Error;

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stream.flush()
    }
}

impl<T: Close, const LEN: usize> Close for SelectorWriteRegistry<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Open, const LEN: usize> Open for SelectorWriteRegistry<T, LEN> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}
