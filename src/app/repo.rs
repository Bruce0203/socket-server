use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use fast_collections::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T> {
    pub index: usize,
    _marker: PhantomData<T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

pub struct Repo<T, const N: usize> {
    elements: Vec<Element<T>, N>,
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Element<T> {
    pub index: Id<T>,
    #[deref]
    #[deref_mut]
    data: T,
}

impl<T, const N: usize> Default for Repo<T, N> {
    fn default() -> Self {
        Self {
            elements: Default::default(),
        }
    }
}

impl<T, const N: usize> Repo<T, N> {
    pub fn new() -> Self {
        Self {
            elements: Vec::uninit(),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn add(&mut self, data: T) -> Result<Id<T>, ()> {
        let id = Id {
            index: self.elements.len(),
            _marker: PhantomData,
        };
        self.elements
            .push(Element { index: id, data })
            .map_err(|_| ())?;
        Ok(id)
    }

    pub fn remove(&mut self, id: Id<T>) {
        unsafe { self.elements.swap_remove_unchecked(id.index) };
    }

    pub fn get(&mut self, id: Id<T>) -> &Element<T> {
        unsafe { self.elements.get_unchecked(id.index) }
    }

    pub fn get_mut(&mut self, id: Id<T>) -> &mut Element<T> {
        unsafe { self.elements.get_unchecked_mut(id.index) }
    }
}

impl<T, const N: usize> Index<Id<T>> for Repo<T, N> {
    type Output = Element<T>;

    fn index(&self, index: Id<T>) -> &Self::Output {
        unsafe { self.elements.get_unchecked(index.index) }
    }
}

impl<T, const N: usize> IndexMut<Id<T>> for Repo<T, N> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        unsafe { self.elements.get_unchecked_mut(index.index) }
    }
}

#[cfg(test)]
mod test {
    use super::Repo;

    #[test]
    fn test() {
        let mut repo: Repo<u8, 10> = Repo::new();
        let id = repo.add(0).unwrap();
        repo.remove(id);
    }
}
