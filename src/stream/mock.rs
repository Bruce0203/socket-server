use std::time::Duration;

use fast_collections::{AddWithIndex, Cursor};

use crate::{
    selector::{SelectableChannel, Selector, SelectorListener},
    Accept, Close, Flush, Id, Read, ReadError, Write,
};

#[derive(Default)]
pub struct MockStream {
    read_buf: Cursor<u8, 1000>,
    write_buf: Cursor<u8, 1000>,
    is_cosed: bool,
}

impl MockStream {
    pub fn flex(&mut self) -> Result<(), ()> {
        let mut temp = Cursor::new();
        temp.push_from_cursor(&mut self.read_buf)?;
        self.read_buf.push_from_cursor(&mut self.write_buf)?;
        self.write_buf = temp;
        Ok(())
    }
}

impl Read for MockStream {
    type Ok = ();
    type Error = ReadError;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        read_buf
            .push_from_cursor(&mut self.write_buf)
            .map_err(|()| ReadError::SocketClosed)
    }
}

impl<const LEN: usize> Write<Cursor<u8, LEN>> for MockStream {
    fn write(&mut self, write_buf: &mut Cursor<u8, LEN>) -> Result<(), Self::Error> {
        self.read_buf.push_from_cursor(write_buf)
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
}

#[derive(Default, derive_more::Deref, derive_more::DerefMut)]
pub struct MockSelector<T, S, const N: usize>(Selector<T, S, N>);

impl<T, S, const N: usize> MockSelector<T, S, N> {
    pub fn new(server: T) -> Self {
        Self(Selector::new(server))
    }
}

impl<T: SelectorListener<SelectableChannel<S>>, const N: usize, S>
    MockSelector<T, SelectableChannel<S>, N>
{
    pub fn entry_point<const LEN: usize>(mut self, _port: u16, timeout: Duration) -> !
    where
        S: Close<Registry = <MockStream as Close>::Registry>
            + Write<Cursor<u8, LEN>>
            + Flush
            + Accept<MockStream>,
    {
        let socket_id = self
            .connections
            .add_with_index(|_i| Accept::accept(MockStream::default()))
            .unwrap();
        let socket_id = Id::from(socket_id);
        loop {
            T::tick(&mut self).unwrap();
            match T::read(&mut self, socket_id.clone()) {
                Ok(()) => {}
                Err(err) => match err {
                    ReadError::NotFullRead => { /*TODO acc rate limit*/ }
                    ReadError::SocketClosed => self.request_socket_close(socket_id.clone()),
                    ReadError::FlushRequest => self.request_socket_flush(socket_id.clone()),
                },
            };
            self.flush_all_sockets(&mut ());
        }
    }
}
