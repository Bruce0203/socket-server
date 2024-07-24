#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]

pub mod mio;

pub mod mock;
pub mod packet_channel;
mod socket;
pub mod socket_id;
pub mod tick_machine;

pub use socket::*;

#[cfg(feature = "websocket")]
pub mod websocket;


