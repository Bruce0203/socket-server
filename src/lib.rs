#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod mio;

pub mod mock;
mod socket;
pub mod tick_machine;

pub use socket::*;

#[cfg(feature = "websocket")]
pub mod websocket;
