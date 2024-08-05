use std::{
    io::{Read, Write},
    mem::{transmute_copy, MaybeUninit},
    net::SocketAddr,
};

use fast_collections::Slab;
use qcell::{LCell, LCellOwner};

use super::socket::{Registry, ServerSocketListener, Socket, SocketState};

pub(crate) trait Poll<T> {
    fn open(&mut self, stream: &mut T, token: usize) -> Result<(), ()>;
    fn close(&mut self, stream: &mut T);
}

pub(crate) struct Selector<'id, 'registry, T, P, Stream>
where
    T: ServerSocketListener<'id>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub poll: P,
    pub server: LCell<'id, T>,
    pub sockets: Slab<Socket<'id, 'registry, T>, { T::MAX_CONNECTIONS }>,
    pub streams: [MaybeUninit<Stream>; T::MAX_CONNECTIONS],
}

impl<'id, 'registry, T, P, Stream> Selector<'id, 'registry, T, P, Stream>
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn new(server: T, owner: &mut LCellOwner<'id>, poll: P) -> Self {
        let streams = MaybeUninit::<[MaybeUninit<Stream>; T::MAX_CONNECTIONS]>::uninit();
        let streams = unsafe { transmute_copy(&streams.assume_init()) };
        Self {
            server: owner.cell(server),
            sockets: Slab::new(),
            streams,
            poll,
        }
    }
}

impl<'id, 'registry, T, P: Poll<Stream>, Stream: Read + Write>
    Selector<'id, 'registry, T, P, Stream>
where
    T: ServerSocketListener<'id, Connection: Default>,
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn read(&mut self, owner: &mut LCellOwner<'id>, token: usize) {
        let socket = unsafe { self.sockets.get_unchecked_mut(token) };
        let stream = unsafe { self.streams.get_unchecked_mut(token).assume_init_mut() };
        match socket.read_buf.rw(owner).push_from_read(stream) {
            Ok(read_len) => {
                if read_len == 0 {
                    socket.register_close_event(owner)
                } else {
                    T::read(owner, &self.server, socket)
                }
            }
            Err(_io_err) => socket.register_close_event(owner),
        }
    }

    pub fn flush_registry(
        &mut self,
        owner: &mut LCellOwner<'id>,
        registry: &'registry LCell<'id, Registry<'id, T>>,
    ) {
        let registry_vec_len = registry.ro(owner).len();
        for ind in 0..registry_vec_len {
            let id = *unsafe { registry.ro(&owner).get_unchecked(ind) };
            let socket = unsafe { self.sockets.get_unchecked_mut(id) };
            let stream = unsafe { self.streams.get_unchecked_mut(id).assume_init_mut() };
            match socket.state {
                SocketState::Idle => continue,
                SocketState::WriteRequest => {
                    socket.state = SocketState::Idle;
                    T::flush(owner, &self.server, socket);
                    match socket.write_buf.rw(owner).push_to_write(stream) {
                        Ok(write_len) => {
                            if write_len == 0 {
                                self.close(owner, id)
                            }
                        }
                        Err(_) => self.close(owner, id),
                    };
                }
                SocketState::CloseRequest => self.close(owner, id),
            }
        }
        registry.rw(owner).clear();
    }

    pub fn accept(
        &mut self,
        owner: &mut LCellOwner<'id>,
        accepted_stream: Stream,
        addr: SocketAddr,
        registry: &'registry LCell<'id, Registry<'id, T>>,
    ) -> Result<(), ()> {
        let id = self
            .sockets
            .add_with_index(|ind| Socket::new(registry, *ind))?;
        let stream = unsafe { self.streams.get_unchecked_mut(id) };
        let socket = unsafe { self.sockets.get_unchecked_mut(id) };
        *stream = MaybeUninit::new(accepted_stream);
        match self
            .poll
            .open(unsafe { stream.assume_init_mut() }, socket.token)
        {
            Ok(()) => T::accept(owner, &self.server, socket, addr),
            Err(_err) => socket.register_close_event(owner),
        }
        Ok(())
    }

    pub fn close(&mut self, owner: &mut LCellOwner<'id>, id: usize) {
        let socket = unsafe { self.sockets.get_unchecked_mut(id) };
        let stream = unsafe { self.streams.get_unchecked_mut(id).assume_init_mut() };
        T::close(owner, &self.server, socket);
        self.poll.close(stream);
        let token = socket.token;
        unsafe { self.sockets.remove_unchecked(token) };
    }
}
