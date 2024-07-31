use fast_collections::Cursor;
use httparse::{Request, EMPTY_HEADER};
use qcell::{LCell, LCellOwner};
use sha1::{Digest, Sha1};

use crate::socket_server::{Socket, SocketListener};

pub enum ReadError {
    NotFullRead,
    FlushRequest,
    CloseRequest,
}
#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum WebSocketState {
    #[default]
    Idle,
    HandShaked,
    Accepted,
}

pub fn websocket_read<'id, const READ_BUFFFER_LEN: usize, const WRITE_BUFFER_LEN: usize>(
    owner: &mut LCellOwner<'id>,
    websocket: &LCell<'id, WebSocketState>,
    read_buf: &LCell<'id, Cursor<u8, { READ_BUFFFER_LEN }>>,
    write_buf: &LCell<'id, Cursor<u8, { WRITE_BUFFER_LEN }>>,
) -> Result<(), ReadError> {
    match websocket.ro(owner) {
        WebSocketState::Idle => {
            let headers = {
                let mut headers = [EMPTY_HEADER; 16];
                let mut request = Request::new(&mut headers);
                request
                    .parse(read_buf.ro(owner).filled())
                    .map_err(|_| ReadError::CloseRequest)?;
                headers
            };
            let key = {
                let key = headers
                    .iter()
                    .find(|e| e.name == "Sec-WebSocket-Key")
                    .ok_or_else(|| ReadError::CloseRequest)?;
                const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
                let mut sha1 = Sha1::default();
                sha1.update(&key.value);
                sha1.update(WS_GUID);
                data_encoding::BASE64.encode(&sha1.finalize())
            };

            let dst = unsafe { write_buf.rw(owner).unfilled_mut() };

            const PREFIX: &[u8; 94] = b"HTTP/1.1 101 Switching Protocols\nUpgrade: websocket\nConnection: Upgrade\nSec-WebSocket-Accept: ";
            const SUFFIX: &[u8; 12] = b"\r\n   \r\n \r\n\r\n";

            const KEY_INDEX: usize = PREFIX.len();
            let key_last_index = KEY_INDEX + key.len();

            dst[..KEY_INDEX].copy_from_slice(PREFIX);
            dst[KEY_INDEX..key_last_index].copy_from_slice(key.as_bytes());
            dst[key_last_index..key_last_index + SUFFIX.len()].copy_from_slice(SUFFIX);

            unsafe {
                *write_buf.rw(owner).filled_len_mut() = write_buf
                    .ro(owner)
                    .filled()
                    .len()
                    .unchecked_add(key_last_index.unchecked_add(SUFFIX.len()))
            };
            read_buf.rw(owner).clear();
            *websocket.rw(owner) = WebSocketState::HandShaked;
            Err(ReadError::FlushRequest)
        }
        WebSocketState::HandShaked => Err(ReadError::CloseRequest),
        WebSocketState::Accepted => {
            let frame_header: u16 = *read_buf
                .rw(owner)
                .read_transmute()
                .ok_or_else(|| ReadError::NotFullRead)?;
            let (header_byte1, header_byte2): (u8, u8) =
                unsafe { fast_collections::const_transmute_unchecked(frame_header) };
            let opcode = header_byte1 & 0b0000_1111;
            if opcode != 2 {
                return Err(ReadError::CloseRequest);
            }
            let mask = header_byte2 & 0b1000_0000;
            let payload_length = header_byte2 & 127;
            const MASK_KEY_LEN: usize = 4;
            if mask != 0 {
                let masking_key = *read_buf
                    .rw(owner)
                    .read_transmute::<[u8; MASK_KEY_LEN]>()
                    .ok_or_else(|| ReadError::NotFullRead)?;
                let mut mask_i = 0;
                let read_cursor_pos = read_buf.ro(owner).pos();
                for i in read_cursor_pos..read_cursor_pos + payload_length as usize {
                    unsafe {
                        *read_buf.rw(owner).get_unchecked_mut(i) =
                            read_buf.ro(owner).get_unchecked(i) ^ masking_key[mask_i]
                    };
                    mask_i += 1;
                    mask_i %= MASK_KEY_LEN;
                }
            }
            Ok(())
        }
    }
}

pub fn websocket_flush<'id, const WRITE_BUFFER_LEN: usize>(
    owner: &mut LCellOwner<'id>,
    websocket: &LCell<'id, WebSocketState>,
    write_buf: &LCell<'id, Cursor<u8, { WRITE_BUFFER_LEN }>>,
) -> Result<(), ()> {
    if *websocket.ro(owner) == WebSocketState::HandShaked {
        *websocket.rw(owner) = WebSocketState::Accepted;
    } else {
        let mut buffer = Cursor::<u8, { WRITE_BUFFER_LEN }>::new();
        let mut write_buf = write_buf.rw(owner);
        let payload = &mut write_buf;
        {
            let header0: u8 = 2;
            buffer.push(header0).map_err(|_| ())?;
        }
        let payload_len = payload.filled_len() - payload.pos();
        if payload_len >= 8 * 8 * 8 {
            let header1: [u8; 8] =
                unsafe { fast_collections::const_transmute_unchecked(payload_len) };
            buffer.push_transmute(header1).map_err(|_| ())?;
        } else if payload_len >= 126 {
            let header1: [u8; 2] =
                unsafe { fast_collections::const_transmute_unchecked(payload_len) };
            buffer.push_transmute(header1).map_err(|_| ())?;
        } else {
            let header1: u8 = payload_len as u8;
            buffer.push(header1).map_err(|_| ())?;
        }
        buffer.push_from_cursor(payload)?;
        write_buf.clear();
        write_buf.push_from_cursor(&mut buffer)?;
    }
    Ok(())
}
