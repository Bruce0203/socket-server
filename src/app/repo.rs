use std::{thread::sleep, time::Duration};

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};
use static_rc::StaticRc;

pub struct Repo<'id, T> {
    elements: Vec<StaticRc<LCell<'id, RepoCell<T>>, 1, 2>, 100>,
}

pub struct RepoCell<T> {
    index: usize,
    data: T,
}

pub struct Id<'id, T> {
    element: StaticRc<LCell<'id, RepoCell<T>>, 1, 2>,
}

impl<'id, T> Id<'id, T> {
    pub fn get<'lt: 'get, 'get, 'owner: 'get>(
        &'lt self,
        owner: &'owner LCellOwner<'id>,
    ) -> &'get T {
        &self.element.ro(owner).data
    }

    pub fn get_mut<'lt: 'get, 'get, 'owner: 'get>(
        &'lt mut self,
        owner: &'owner mut LCellOwner<'id>,
    ) -> &'get mut T {
        &mut self.element.rw(owner).data
    }
}

impl<'id, T> Repo<'id, T> {
    pub fn new() -> Self {
        Self {
            elements: Vec::uninit(),
        }
    }

    pub fn add(&mut self, owner: &mut LCellOwner<'id>, data: T) -> Result<Id<'id, T>, ()> {
        let index = self.elements.len();
        let element = StaticRc::new(owner.cell(RepoCell { index, data }));
        let (rc1, rc2) = StaticRc::split::<1, 1>(element);
        self.elements.push(rc1).map_err(|_| ())?;
        let id = Id { element: rc2 };
        Ok(id)
    }

    pub fn remove(&mut self, owner: &mut LCellOwner<'id>, id: Id<'id, T>) {
        let ind = id.element.ro(owner).index;
        sleep(Duration::from_secs(4));
        let removed = unsafe { self.elements.swap_remove_unchecked(ind) };
        let joined: StaticRc<LCell<'id, RepoCell<T>>, 2, 2> = StaticRc::join(removed, id.element);
        drop(joined);
    }
}

#[cfg(test)]
mod test {
    use qcell::LCellOwner;

    use super::Repo;

    #[test]
    fn test() {
        LCellOwner::scope(|mut owner| {
            struct Token(usize);
            let mut repo: Repo<Token> = Repo::new();
            let mut id = repo.add(&mut owner, Token(123)).unwrap();
            let token = id.get_mut(&mut owner);
            repo.remove(&mut owner, id);
        });
    }
}
