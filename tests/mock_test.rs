#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::time::Duration;

use qcell::{LCell, LCellOwner};
#[cfg(test)]
use socket_server::socket::{ServerSocketListener, Socket};

#[test]
fn test_mocking_system() {
    LCellOwner::scope(|mut owner| {
        #[cfg(test)]
        socket_server::mock::run_mock(
            &mut owner,
            ApplicationServer {},
            MockServer {},
            Duration::from_millis(50),
        )
    })
}

pub struct ApplicationServer {}
#[derive(Default)]
pub struct Player {}

impl<'id> ServerSocketListener<'id> for ApplicationServer {
    const MAX_CONNECTIONS: usize = 10;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    type Connection = Player;

    fn tick(_server: &LCell<'id, Self>, _owner: &mut LCellOwner<'id>) {}

    fn accept(
        owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        connection: &mut Socket<'id, '_, Self>,
        _addr: std::net::SocketAddr,
    ) {
        connection.register_close_event(owner)
    }

    fn read(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
    }

    fn flush(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
    }

    fn close(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
    }
}

pub struct MockServer {}
#[derive(Default)]
pub struct MockPlayer {}

impl<'id> ServerSocketListener<'id> for MockServer {
    const MAX_CONNECTIONS: usize = 10;
    const READ_BUFFFER_LEN: usize = 100;
    const WRITE_BUFFER_LEN: usize = 100;
    type Connection = MockPlayer;

    fn tick(_server: &LCell<'id, Self>, _owner: &mut LCellOwner<'id>) {}

    fn accept(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
        _addr: std::net::SocketAddr,
    ) {
    }

    fn read(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
    }

    fn flush(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
    }

    fn close(
        _owner: &mut LCellOwner<'id>,
        _server: &LCell<'id, Self>,
        _connection: &mut Socket<'id, '_, Self>,
    ) {
        todo!()
    }
}
