#![feature(try_blocks)]
#![feature(generic_arg_infer)]

pub mod connection;
pub mod selector;
pub mod stream;
pub mod tick_machine;

use std::marker::PhantomData;

use fast_collections::{Cursor, GetUnchecked, Slab};
use mio::Registry;
use nonmax::NonMaxUsize;

pub trait Read {
    type Ok;
    type Error;
    fn read<const N: usize>(
        &mut self,
        read_buf: &mut Cursor<u8, N>,
    ) -> Result<Self::Ok, Self::Error>;
}

#[derive(Debug)]
pub enum ReadError {
    NotFullRead,
    FlushRequest,
    SocketClosed,
}

pub trait Write<T>: Flush {
    fn write(&mut self, write: &mut T) -> Result<(), Self::Error>;
}

pub trait Flush {
    type Error;
    fn flush(&mut self) -> Result<(), Self::Error>;
}

pub trait Close {
    type Error;
    type Registry;
    fn is_closed(&self) -> bool;
    fn close(&mut self, registry: &mut Self::Registry) -> Result<(), Self::Error>;
}

pub trait Open {
    type Error;
    type Registry;
    fn open(&mut self, registry: &mut Registry) -> Result<(), Self::Error>;
}

pub trait Accept<T>: Sized {
    fn accept(accept: T) -> Self;
}

pub struct Repo<T> {
    elements: Slab<Id<T>, T, 100>,
}

#[repr(C)]
#[derive(Debug)]
pub struct Id<T> {
    inner: NonMaxUsize,
    _marker: PhantomData<T>,
}

const _: () = {
    if size_of::<Id<()>>() != size_of::<usize>() {
        panic!("size of id is not same as usize")
    }
};

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Into<usize> for &Id<T> {
    fn into(self) -> usize {
        self.inner.get()
    }
}

impl<T> From<usize> for Id<T> {
    fn from(value: usize) -> Self {
        Self {
            inner: unsafe { NonMaxUsize::new_unchecked(value) },
            _marker: PhantomData,
        }
    }
}

impl<T> Repo<T> {
    pub fn get_mut(&mut self, id: &Id<T>) -> &mut T {
        unsafe { self.elements.get_unchecked_mut(id) }
    }

    pub fn get(&mut self, id: &Id<T>) -> &T {
        unsafe { self.elements.get_unchecked(id) }
    }
}
