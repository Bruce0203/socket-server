use fast_collections::Cursor;

use crate::{Read, Stream, Write};

pub struct MockStream {
    pub read_buffer: Cursor<u8, 100>,
    pub write_buffer: Cursor<u8, 100>,
}

impl Stream for MockStream {
    type Error = ();
    type Registry = ();

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        Ok(())
    }

    fn open(&mut self, token: usize, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Read for MockStream {
    type Error = ();

    fn read<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        buffer.push_from_cursor(&mut self.write_buffer)?;
        Ok(())
    }
}

impl Write for MockStream {
    type Error = ();

    fn write<const N: usize>(&mut self, buffer: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.read_buffer.push_from_cursor(buffer)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
