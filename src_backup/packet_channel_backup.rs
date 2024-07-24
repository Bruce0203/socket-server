use std::marker::PhantomData;

use fast_collections::{Clear, Cursor, GetUnchecked};
use packetize::{ClientBoundPacketStream, ServerBoundPacketStream};

use crate::{ServerSocketService, SocketEvents, SocketId};

#[derive(Default)]
pub struct ClientBoundPacketChannel<T, E> {
    state: T,
    ext: E,
}

#[derive(Default)]
pub struct ServerBoundPacketChannel<T, E> {
    state: T,
    ext: E,
}

fn handle_read<T: ServerSocketService, F, D>(
    service: &mut T,
    socket_id: &SocketId,
    registry: &mut SocketEvents<T>,
    handle_read: F,
    mut decode_packet: D,
) -> Result<(), ()>
where
    D: FnMut(&mut T, &mut Cursor<u8, { T::READ_BUFFER_LENGTH }>) -> Result<(), ()>,
    F: FnMut((), &mut crate::SocketEvents<T>) -> Result<(), ()>,
    [(); T::MAX_CONNECTIONS]:,
    [(); T::READ_BUFFER_LENGTH]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
{
    let socket = unsafe { registry.connections.get_unchecked_mut(socket_id) };
    while { socket.read_buffer.remaining().clone() } != 0 {
        let packet = decode_packet(service, &mut socket.read_buffer).map_err(|_| ())?;
        //handle_read(packet)?;
    }
    socket.read_buffer.clear();
    Ok(())
}

impl<T: ServerSocketService> SocketEvents<T>
where
    [(); T::MAX_CONNECTIONS]:,
    [(); T::WRITE_BUFFER_LENGTH]:,
    [(); T::READ_BUFFER_LENGTH]:,
{
    fn a() {}
}
