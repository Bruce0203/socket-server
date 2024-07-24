use fast_collections::Cursor;

use crate::{Accept, Close, Flush, Read, ReadError, Write};

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
