use crate::{Accept, Close, Flush, Open, Read, Write};
use fast_collections::Cursor;

pub struct MioTcpStream {
    pub stream: mio::net::TcpStream,
    pub token: mio::Token,
    pub is_closed: bool,
}

impl Read for MioTcpStream {
    type Ok = ();
    type Error = std::io::Error;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        read_buf.push_from_read(&mut self.stream)?;
        Ok(())
    }
}

impl Accept<MioTcpStream> for MioTcpStream {
    fn accept(accept: MioTcpStream) -> Self {
        accept
    }
}

impl<const N: usize> Write<Cursor<u8, N>> for MioTcpStream {
    fn write(&mut self, write_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        write_buf.push_to_write(&mut self.stream)
    }
}

impl Flush for MioTcpStream {
    type Error = std::io::Error;
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Close for MioTcpStream {
    type Error = std::io::Error;

    type Registry = mio::Registry;

    fn close(&mut self, registry: &mut mio::Registry) -> Result<(), std::io::Error> {
        mio::event::Source::deregister(&mut self.stream, registry)
    }

    fn is_closed(&self) -> bool {
        self.is_closed
    }
}

impl Open for MioTcpStream {
    type Error = std::io::Error;

    type Registry = mio::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        mio::event::Source::register(
            &mut self.stream,
            registry,
            self.token,
            mio::Interest::READABLE,
        )
    }
}
