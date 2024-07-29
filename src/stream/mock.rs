use fast_collections::{AddWithIndex, Cursor};

use super::{Accept, Close, Flush, Id, Read, ReadError, Write};
use crate::selector::{Selector, SelectorListener};

use super::readable_byte_channel::PollRead;

#[derive(Default)]
pub struct MockStream<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> {
    pub stream_read_buf: Cursor<u8, READ_BUF_LEN>,
    pub stream_write_buf: Cursor<u8, WRITE_BUF_LEN>,
    pub is_cosed: bool,
}

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize>
    MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
    pub fn flex(&mut self, stream: &mut Self) -> Result<(), ()> {
        stream
            .stream_write_buf
            .push_from_cursor(&mut self.stream_read_buf)?;
        self.stream_write_buf
            .push_from_cursor(&mut stream.stream_read_buf)?;
        Ok(())
    }
}

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> Read
    for MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), ReadError> {
        read_buf
            .push_from_cursor(&mut self.stream_write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> Write
    for MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
    fn write<const LEN: usize>(
        &mut self,
        write_buf: &mut Cursor<u8, LEN>,
    ) -> Result<(), Self::Error> {
        self.stream_read_buf.push_from_cursor(write_buf)
    }
}

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> Flush
    for MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
    type Error = ();

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> Close
    for MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
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

impl<const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize>
    Accept<MockStream<READ_BUF_LEN, WRITE_BUF_LEN>> for MockStream<READ_BUF_LEN, WRITE_BUF_LEN>
{
    fn accept(accept: MockStream<READ_BUF_LEN, WRITE_BUF_LEN>) -> Self {
        accept
    }

    fn get_stream(&mut self) -> &mut MockStream<READ_BUF_LEN, WRITE_BUF_LEN> {
        self
    }
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct MockSelector<T, const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize>(T);

impl<T: Default, const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize> Default
    for MockSelector<T, READ_BUF_LEN, WRITE_BUF_LEN>
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, const READ_BUF_LEN: usize, const WRITE_BUF_LEN: usize>
    MockSelector<T, READ_BUF_LEN, WRITE_BUF_LEN>
{
    pub fn new(server: T) -> Self {
        Self(server)
    }
}

impl<
        T: SelectorListener<S, C, N>,
        S,
        C: Default,
        const N: usize,
        const READ_BUF_LEN: usize,
        const WRITE_BUF_LEN: usize,
    > MockSelector<Selector<T, S, C, N>, READ_BUF_LEN, WRITE_BUF_LEN>
{
    pub fn entry_point<T2: SelectorListener<S2, C2, N2>, S2, C2: Default, const N2: usize>(
        mut self,
        mut server: MockSelector<Selector<T2, S2, C2, N2>, READ_BUF_LEN, WRITE_BUF_LEN>,
    ) where
        S: Close<Registry = <MockStream<READ_BUF_LEN, WRITE_BUF_LEN> as Close>::Registry>
            + Flush
            + Accept<MockStream<READ_BUF_LEN, WRITE_BUF_LEN>>
            + PollRead,
        S2: Close<Registry = <MockStream<READ_BUF_LEN, WRITE_BUF_LEN> as Close>::Registry>
            + Flush
            + Accept<MockStream<READ_BUF_LEN, WRITE_BUF_LEN>>
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
        T2::accept(&mut server.0, id2.clone());
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
