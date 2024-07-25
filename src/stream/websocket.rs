use fast_collections::{Clear, Cursor, CursorReadTransmute, GetUnchecked, Push, PushTransmute};
use httparse::{Request, EMPTY_HEADER};
use sha1::{Digest, Sha1};

use crate::{Accept, Close, Flush, Open, Read, ReadError, Write};

use super::writable_byte_channel::WritableByteChannel;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
enum WebSocketState {
    #[default]
    Idle,
    HandShaked,
    Accepted,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct WebSocketServer<T> {
    #[deref]
    #[deref_mut]
    pub stream: T,
    state: WebSocketState,
}

impl<T> From<T> for WebSocketServer<T> {
    fn from(value: T) -> Self {
        Self {
            stream: value,
            state: WebSocketState::default(),
        }
    }
}

impl<T: Write<Cursor<u8, WRITE_BUF_LEN>> + Read<()>, const WRITE_BUF_LEN: usize> Read<()>
    for WebSocketServer<WritableByteChannel<T, WRITE_BUF_LEN>>
{
    type Error = ReadError;

    fn read<const N: usize>(&mut self, read_buf: &mut Cursor<u8, N>) -> Result<(), Self::Error> {
        self.stream
            .read(read_buf)
            .map_err(|_| ReadError::SocketClosed)?;
        match self.state {
            WebSocketState::Idle => {
                let headers = {
                    let mut headers = [EMPTY_HEADER; 16];
                    let mut request = Request::new(&mut headers);
                    request
                        .parse(read_buf.filled())
                        .map_err(|_| ReadError::SocketClosed)?;
                    headers
                };
                let key = {
                    let key = headers
                        .iter()
                        .find(|e| e.name == "Sec-WebSocket-Key")
                        .ok_or_else(|| ReadError::SocketClosed)?;
                    const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
                    let mut sha1 = Sha1::default();
                    sha1.update(&key.value);
                    sha1.update(WS_GUID);
                    data_encoding::BASE64.encode(&sha1.finalize())
                };

                let dst = unsafe { self.stream.write_buf.unfilled_mut() };

                const PREFIX: &[u8; 94] = b"HTTP/1.1 101 Switching Protocols\nUpgrade: websocket\nConnection: Upgrade\nSec-WebSocket-Accept: ";
                const SUFFIX: &[u8; 12] = b"\r\n   \r\n \r\n\r\n";

                const KEY_INDEX: usize = PREFIX.len();
                let key_last_index = KEY_INDEX + key.len();

                dst[..KEY_INDEX].copy_from_slice(PREFIX);
                dst[KEY_INDEX..key_last_index].copy_from_slice(key.as_bytes());
                dst[key_last_index..key_last_index + SUFFIX.len()].copy_from_slice(SUFFIX);

                unsafe {
                    *self.stream.write_buf.filled_len_mut() = self
                        .stream
                        .write_buf
                        .filled()
                        .len()
                        .unchecked_add(key_last_index.unchecked_add(SUFFIX.len()))
                };
                read_buf.clear();
                self.state = WebSocketState::HandShaked;
                Err(ReadError::FlushRequest)
            }
            WebSocketState::HandShaked => Err(ReadError::SocketClosed),
            WebSocketState::Accepted => {
                let frame_header: u16 = *read_buf
                    .read_transmute()
                    .ok_or_else(|| ReadError::NotFullRead)?;
                let (header_byte1, header_byte2): (u8, u8) =
                    unsafe { fast_collections::const_transmute_unchecked(frame_header) };
                let opcode = header_byte1 & 0b0000_1111;
                if opcode != 2 {
                    return Err(ReadError::SocketClosed);
                }
                let mask = header_byte2 & 0b1000_0000;
                let payload_length = header_byte2 & 127;
                const MASK_KEY_LEN: usize = 4;
                if mask != 0 {
                    let masking_key = *read_buf
                        .read_transmute::<[u8; MASK_KEY_LEN]>()
                        .ok_or_else(|| ReadError::NotFullRead)?;
                    let mut mask_i = 0;
                    let read_cursor_pos = read_buf.pos();
                    for i in read_cursor_pos..read_cursor_pos + payload_length as usize {
                        unsafe {
                            *read_buf.get_unchecked_mut(i) =
                                read_buf.get_unchecked(i) ^ masking_key[mask_i]
                        };
                        mask_i += 1;
                        mask_i %= MASK_KEY_LEN;
                    }
                }
                //self.server.poll_read(socket_id, registry);
                Ok(())
            }
        }
    }
}

impl<T: Write<Cursor<u8, WRITE_BUF_LEN>>, const WRITE_BUF_LEN: usize>
    Write<Cursor<u8, WRITE_BUF_LEN>> for WebSocketServer<WritableByteChannel<T, WRITE_BUF_LEN>>
{
    fn write(&mut self, write_buf: &mut Cursor<u8, WRITE_BUF_LEN>) -> Result<(), Self::Error> {
        self.stream.write(write_buf).map_err(|_| ())
    }
}

impl<T: Flush + Write<Cursor<u8, LEN>>, const LEN: usize> Flush
    for WebSocketServer<WritableByteChannel<T, LEN>>
{
    type Error = ();

    fn flush(&mut self) -> Result<(), Self::Error> {
        if self.state == WebSocketState::HandShaked {
            self.state = WebSocketState::Accepted;
            self.stream.flush().map_err(|_| ())?;
        } else {
            let mut buffer = Cursor::<u8, LEN>::new();
            let payload = &mut self.stream.write_buf;
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
            self.stream.write_buf.clear();
            self.write(&mut buffer)?;
            self.stream.flush().map_err(|_| ())?;
        }
        Ok(())
    }
}

impl<T: Accept<A>, A> Accept<A> for WebSocketServer<T> {
    fn accept(accept: A) -> Self {
        Self {
            stream: T::accept(accept),
            state: WebSocketState::default(),
        }
    }

    fn get_stream(&mut self) -> &mut A {
        self.stream.get_stream()
    }
}

impl<T: Close> Close for WebSocketServer<T> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error> {
        self.stream.close(registry)
    }
}

impl<T: Open> Open for WebSocketServer<T> {
    type Error = T::Error;

    type Registry = T::Registry;

    fn open(&mut self, registry: &mut mio::Registry) -> Result<(), Self::Error> {
        self.stream.open(registry)
    }
}
