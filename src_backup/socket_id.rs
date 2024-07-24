use nonmax::NonMaxUsize;

pub struct SocketId(NonMaxUsize);

impl SocketId {
    pub(super) fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    pub(crate) fn from(value: usize) -> Self {
        Self(unsafe { NonMaxUsize::new_unchecked(value) })
    }
}

impl Into<usize> for &SocketId {
    fn into(self) -> usize {
        self.0.get()
    }
}
