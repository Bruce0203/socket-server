use crate::{Accept, Close, Flush, Open, Write};

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct ConnectionPipe<T, S> {
    stream: T,
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
