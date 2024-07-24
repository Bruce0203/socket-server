use fast_collections::{
    const_transmute_unchecked, Clear, Cursor, CursorReadTransmute, GetUnchecked, Push,
    PushTransmute,
};
use httparse::{Request, EMPTY_HEADER};
use sha1::{Digest, Sha1};

use crate::{ServerSocketService, SocketEvents, SocketId};

#[derive(Default)]
pub struct WebSocketChannel {
    is_connection_established: bool,
}

impl<T: ServerSocketService> ReadChannel<T> for WebSocketChannel {
    type Read = ();
    fn on_read<F>(
        socket_id: &SocketId,
        registry: &mut SocketEvents<T>,
        _handle_read: F,
    ) -> Result<(), SocketReadError>
    where
        F: FnMut(Self::Read, &mut SocketEvents<T>) -> Result<(), SocketReadError>,
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:,
    {
        let socket = unsafe { registry.connections.get_unchecked_mut(socket_id) };
        if socket.channel.is_connection_established {
            let frame_header: u16 = *socket
                .read_buffer
                .read_transmute()
                .ok_or(SocketReadError::NotFullRead)?;
            let (header_byte1, header_byte2): (u8, u8) =
                unsafe { const_transmute_unchecked(frame_header) };
            let opcode = header_byte1 & 0b0000_1111;
            if opcode != 2 {
                return Err(SocketReadError::SocketClosed);
            }
            let mask = header_byte2 & 0b1000_0000;
            let payload_length = header_byte2 & 127;
            const MASK_KEY_LEN: usize = 4;
            if mask != 0 {
                let masking_key = *socket
                    .read_buffer
                    .read_transmute::<[u8; MASK_KEY_LEN]>()
                    .ok_or_else(|| SocketReadError::NotFullRead)?;
                let mut mask_i = 0;
                let read_cursor_pos = socket.read_buffer.pos();
                for i in read_cursor_pos..read_cursor_pos + payload_length as usize {
                    unsafe {
                        *socket.read_buffer.get_unchecked_mut(i) =
                            socket.read_buffer.get_unchecked(i) ^ masking_key[mask_i]
                    };
                    mask_i += 1;
                    mask_i %= MASK_KEY_LEN;
                }
            }
            Ok(())
        } else {
            let headers = {
                let mut headers = [EMPTY_HEADER; 16];
                let mut request = Request::new(&mut headers);
                request
                    .parse(socket.read_buffer.filled())
                    .map_err(|_| SocketReadError::SocketClosed)?;
                headers
            };
            let key = {
                let key = headers
                    .iter()
                    .find(|e| e.name == "Sec-WebSocket-Key")
                    .ok_or_else(|| SocketReadError::SocketClosed)?;
                const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
                let mut sha1 = Sha1::default();
                sha1.update(&key.value);
                sha1.update(WS_GUID);
                data_encoding::BASE64.encode(&sha1.finalize())
            };

            let dst = unsafe { socket.write_buffer.unfilled_mut() };

            const PREFIX: &[u8; 94] = b"HTTP/1.1 101 Switching Protocols\nUpgrade: websocket\nConnection: Upgrade\nSec-WebSocket-Accept: ";
            const SUFFIX: &[u8; 12] = b"\r\n   \r\n \r\n\r\n";

            const KEY_INDEX: usize = PREFIX.len();
            let key_last_index = KEY_INDEX + key.len();

            dst[..KEY_INDEX].copy_from_slice(PREFIX);
            dst[KEY_INDEX..key_last_index].copy_from_slice(key.as_bytes());
            dst[key_last_index..key_last_index + SUFFIX.len()].copy_from_slice(SUFFIX);

            unsafe {
                *socket.write_buffer.filled_len_mut() = socket
                    .write_buffer
                    .filled()
                    .len()
                    .unchecked_add(key_last_index.unchecked_add(SUFFIX.len()))
            };
            socket.read_buffer.clear();
            socket.channel.is_connection_established = true;
            registry.register_write_event(socket_id);
            Err(SocketReadError::NotFullRead)
        }
    }
}

impl WriteChannel for WebSocketChannel {
    type Write = ();

    fn on_write<T: ServerSocketService>(
        &mut self,
        _buffer: &mut Cursor<u8, { T::WRITE_BUFFER_LENGTH }>,
        _write: &Self::Write,
    ) -> Result<(), ()>
    where
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:,
    {
        Ok(())
    }

    fn on_flush<T: ServerSocketService<Channel = Self>>(
        socket_id: &SocketId,
        registry: &mut SocketEvents<T>,
    ) -> Result<(), ()>
    where
        [(); T::MAX_CONNECTIONS]:,
        [(); T::READ_BUFFER_LENGTH]:,
        [(); T::WRITE_BUFFER_LENGTH]:,
    {
        let socket = unsafe { registry.connections.get_unchecked_mut(socket_id) };
        if !socket.channel.is_connection_established {
            let mut buffer = Cursor::<u8, { T::WRITE_BUFFER_LENGTH }>::new();
            let payload = &mut socket.write_buffer;
            {
                let header0: u8 = 2;
                buffer.push(header0).map_err(|_| ())?;
            }
            let payload_len = payload.filled_len() - payload.pos();
            if payload_len >= 8 * 8 * 8 {
                let header1: [u8; 8] = unsafe { const_transmute_unchecked(payload_len) };
                buffer.push_transmute(header1).map_err(|_| ())?;
            } else if payload_len >= 126 {
                let header1: [u8; 2] = unsafe { const_transmute_unchecked(payload_len) };
                buffer.push_transmute(header1).map_err(|_| ())?;
            } else {
                let header1: u8 = payload_len as u8;
                buffer.push(header1).map_err(|_| ())?;
            }
            buffer.push_from_cursor(payload)?;
            socket.write_buffer = buffer;
        }
        Ok(())
    }
}
