use fast_collections::Cursor;

use crate::{Close, Read, Write};

#[derive(Default)]
pub struct MockStream {
    read_buf: Cursor<u8, 1000>,
    write_buf: Cursor<u8, 1000>,
    is_cosed: bool,
}

impl Read for MockStream {
    type Ok = ();
    type Error = ();

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        read_buf.push_from_cursor(&mut self.write_buf)
    }
}

impl Write for MockStream {
    type Error = ();

    fn write<const N: usize>(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.read_buf.push_from_cursor(write_buf)
    }

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
