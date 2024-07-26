use fast_collections::{AddWithIndex, Cursor};

use super::{Accept, Close, Flush, Id, Read, ReadError, Write};
use crate::selector::{Selector, SelectorListener};

use super::readable_byte_channel::PollRead;

#[derive(Default)]
pub struct MockStream {
    pub stream_read_buf: Cursor<u8, 1000>,
    pub stream_write_buf: Cursor<u8, 1000>,
    pub is_cosed: bool,
}

impl MockStream {
    pub fn flex(&mut self, stream: &mut Self) -> Result<(), ()> {
        stream
            .stream_write_buf
            .push_from_cursor(&mut self.stream_read_buf)?;
        self.stream_write_buf
            .push_from_cursor(&mut stream.stream_read_buf)?;
        Ok(())
    }
}

impl Read for MockStream {
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        read_buf
            .push_from_cursor(&mut self.stream_write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl Write for MockStream {
    fn write<const LEN: usize>(
        &mut self,
        write_buf: &mut Cursor<u8, LEN>,
    ) -> Result<(), Self::Error> {
        self.stream_read_buf.push_from_cursor(write_buf)
    }
}

impl Flush for MockStream {
    type Error = ();

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Close for MockStream {
    type Error = ();

    type Registry = ();

    fn close(&mut self, _registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.is_cosed = true;
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.is_cosed
    }
}

impl Accept<MockStream> for MockStream {
    fn accept(accept: MockStream) -> Self {
        accept
    }

    fn get_stream(&mut self) -> &mut MockStream {
        self
    }
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct MockSelector<T, S, C, const N: usize>(Selector<T, S, C, N>);

impl<T: Default, S, C, const N: usize> Default for MockSelector<T, S, C, N> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, S, C, const N: usize> MockSelector<T, S, C, N> {
    pub fn new(server: T) -> Self {
        Self(Selector::new(server))
    }
}

impl<T: SelectorListener<S, C>, C: Default, const N: usize, S> MockSelector<T, S, C, N> {
    pub fn entry_point<
        T2: SelectorListener<S2, C2>,
        S2: Close + Flush,
        C2: Default,
        const N2: usize,
    >(
        mut self,
        mut server: MockSelector<T2, S2, C2, N2>,
    ) where
        S: Close<Registry = <MockStream as Close>::Registry>
            + Flush
            + Accept<MockStream>
            + PollRead,
        S2: Close<Registry = <MockStream as Close>::Registry>
            + Flush
            + Accept<MockStream>
            + PollRead,
    {
        let id = unsafe {
            Id::from(
                self.connections
                    .add_with_index(|_i| Accept::accept(MockStream::default()))
                    .unwrap(),
            )
        };
        T::accept(&mut self, id.clone());
        let id2 = unsafe {
            Id::from(
                server
                    .connections
                    .add_with_index(|_i| Accept::accept(MockStream::default()))
                    .unwrap(),
            )
        };
        T2::accept(&mut server, id2.clone());
        loop {
            let socket = self.get_mut(&id);
            let socket2 = server.get_mut(&id2);
            socket.get_stream().flex(socket2.get_stream()).unwrap();
            if socket.is_closed() || socket2.is_closed() {
                break;
            }
            T::tick(&mut self).unwrap();
            T2::tick(&mut server).unwrap();

            let socket = self.get_mut(&id);
            let socket2 = server.get_mut(&id2);
            socket.get_stream().flex(socket2.get_stream()).unwrap();
            if socket.get_stream().stream_write_buf.remaining() != 0 {
                self.handle_read(id.clone(), &mut ());
            }
            if socket2.get_stream().stream_write_buf.remaining() != 0 {
                server.handle_read(id2.clone(), &mut ());
            }
            self.flush_all_sockets(&mut ());
            server.flush_all_sockets(&mut ());
        }
    }
}
