use std::time::Duration;

use qcell::LCellOwner;

use crate::socket_server::{Socket, SocketListener};

use super::container::Container;

#[derive(Default)]
pub struct Connection {}

impl<'id> SocketListener<'id> for Container<'id, '_, '_> {
    const MAX_CONNECTIONS: usize = 5000;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    const TICK: Duration = Duration::from_millis(50);
    type Connection = Connection;

    fn tick(&mut self, owner: &mut LCellOwner<'id>) {}

    fn accept(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {}

    fn read(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {}

    fn flush(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {}

    fn close(&mut self, owner: &mut LCellOwner<'id>, connection: &mut Socket<'id, '_, Self>) {}
}
