use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use fast_collections::Cursor;
use qcell::LCellOwner;

use crate::{
    selector::{Poll, Selector},
    socket::{Registry, ServerSocketListener, SocketState},
    tick_machine::TickMachine,
};

pub(self) struct MockPoll;

impl<'id, T> Poll<MockStream<'id, T>> for MockPoll
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
{
    fn open(&mut self, _stream: &mut MockStream<'id, T>, _token: usize) -> Result<(), ()> {
        Ok(())
    }

    fn close(&mut self, _stream: &mut MockStream<'id, T>) {}
}

pub(self) struct MockStream<'id, T>
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
{
    read_buf: Cursor<u8, { T::READ_BUFFFER_LEN }>,
    write_buf: Cursor<u8, { T::WRITE_BUFFER_LEN }>,
}

impl<'id, T> Read for MockStream<'id, T>
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut self.write_buf, buf)
    }
}

impl<'id, T> Write for MockStream<'id, T>
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut self.read_buf, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Write::flush(&mut self.read_buf)
    }
}

impl<'id, T> MockStream<'id, T>
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
{
    pub fn new() -> Self {
        Self {
            read_buf: Default::default(),
            write_buf: Default::default(),
        }
    }

    pub fn flex<S>(&mut self, stream: &mut MockStream<'id, S>) -> Result<(), ()>
    where
        S: ServerSocketListener<'id>,
        [(); S::READ_BUFFFER_LEN]:,
        [(); S::WRITE_BUFFER_LEN]:,
    {
        stream.write_buf.push_from_cursor(&mut self.read_buf)?;
        self.write_buf.push_from_cursor(&mut stream.read_buf)?;
        Ok(())
    }
}

pub fn run_mock<'id, T1: ServerSocketListener<'id>, T2: ServerSocketListener<'id>>(
    owner: &mut LCellOwner<'id>,
    server1: T1,
    server2: T2,
    tick: Duration,
) where
    T1: ServerSocketListener<'id, Connection: Default>,
    [(); T1::MAX_CONNECTIONS]:,
    [(); T1::READ_BUFFFER_LEN]:,
    [(); T1::WRITE_BUFFER_LEN]:,
    T2: ServerSocketListener<'id, Connection: Default>,
    [(); T2::MAX_CONNECTIONS]:,
    [(); T2::READ_BUFFFER_LEN]:,
    [(); T2::WRITE_BUFFER_LEN]:,
{
    const ZERO_ADDR: SocketAddr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
    let mut selector1 = Selector::<_, _, MockStream<T1>>::new(server1, owner, MockPoll);
    let mut selector2 = Selector::<_, _, MockStream<T2>>::new(server2, owner, MockPoll);
    let mut tick_machine = TickMachine::new(tick);
    let registry1 = owner.cell(Registry::<'id, T1>::new());
    let registry2 = owner.cell(Registry::<'id, T2>::new());
    selector1
        .accept(owner, MockStream::new(), ZERO_ADDR, &registry1)
        .unwrap();
    selector2
        .accept(owner, MockStream::new(), ZERO_ADDR, &registry2)
        .unwrap();
    loop {
        let socket1 = unsafe { selector1.sockets.get_unchecked_mut(0) };
        let socket2 = unsafe { selector2.sockets.get_unchecked_mut(0) };
        let stream1 = unsafe { selector1.streams.get_unchecked_mut(0).assume_init_mut() };
        let stream2 = unsafe { selector2.streams.get_unchecked_mut(0).assume_init_mut() };

        MockStream::flex(stream1, stream2).unwrap();
        if socket1.state == SocketState::CloseRequest || socket2.state == SocketState::CloseRequest
        {
            break;
        }
        tick_machine.tick(|| {
            T1::tick(&selector1.server, owner);
            T2::tick(&selector2.server, owner);
        });
        MockStream::flex(stream1, stream2).unwrap();
        if stream1.write_buf.remaining() != 0 {
            selector1.read(owner, 0);
        }
        if stream2.write_buf.remaining() != 0 {
            selector2.read(owner, 0);
        }

        selector1.flush_registry(owner, &registry1);
        selector2.flush_registry(owner, &registry2);
    }
}
