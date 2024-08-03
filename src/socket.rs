use derive_more::{Deref, DerefMut};
use fast_collections::{Cursor, Vec};
use qcell::{LCell, LCellOwner};
use std::time::Duration;

#[derive(Deref, DerefMut)]
pub struct Socket<'id: 'registry, 'registry, T: ServerSocketListener<'id>>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    #[deref]
    #[deref_mut]
    pub connection: T::Connection,
    pub read_buf: LCell<'id, Cursor<u8, { T::READ_BUFFFER_LEN }>>,
    pub write_buf: LCell<'id, Cursor<u8, { T::WRITE_BUFFER_LEN }>>,
    pub(crate) state: SocketState,
    pub(crate) token: usize,
    pub(crate) registry: &'registry LCell<'id, Registry<'id, T>>,
}

#[derive(Deref, DerefMut)]
pub(crate) struct Registry<'id, T: ServerSocketListener<'id>>
where
    [(); T::MAX_CONNECTIONS]:,
{
    pub(crate) vec: Vec<usize, { T::MAX_CONNECTIONS }>,
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SocketState {
    #[default]
    Idle,
    WriteRequest,
    CloseRequest,
}

impl<'id, 'registry, T: ServerSocketListener<'id>> Socket<'id, 'registry, T>
where
    [(); T::READ_BUFFFER_LEN]:,
    [(); T::WRITE_BUFFER_LEN]:,
    [(); T::MAX_CONNECTIONS]:,
{
    pub fn register_flush_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::WriteRequest;
    }

    pub fn register_close_event(&mut self, owner: &mut LCellOwner<'id>) {
        self.register_event(owner);
        self.state = SocketState::CloseRequest;
    }

    pub(self) fn register_event(&mut self, owner: &mut LCellOwner<'id>) {
        if self.state == SocketState::Idle {
            let registry = owner.rw(self.registry);
            unsafe { registry.push_unchecked(self.token) };
        }
    }
}

pub trait ServerSocketListener<'id>: Sized {
    const MAX_CONNECTIONS: usize;
    const READ_BUFFFER_LEN: usize;
    const WRITE_BUFFER_LEN: usize;
    const TICK: Duration;
    type Connection;

    fn tick(&mut self, owner: &mut LCellOwner<'id>);

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn read(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn flush(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;

    fn close(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>)
    where
        [(); Self::READ_BUFFFER_LEN]:,
        [(); Self::WRITE_BUFFER_LEN]:,
        [(); Self::MAX_CONNECTIONS]:;
}
