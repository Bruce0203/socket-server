#![feature(generic_arg_infer)]

use std::usize;

use fast_collections::Vec;
use qcell::{LCell, LCellOwner};
use static_rc::StaticRc;

#[test]
fn a() {
    LCellOwner::scope(|mut owner| {});
}

fn asdf<'id>(owner: &mut LCellOwner<'id>) {
    let registry = Registry {};
    let registry = owner.cell(registry);
    pub type T<'id> = LCell<'id, Registry>;
    let registry = StaticRc::<T, 100, 100>::new(registry);
    let arr = StaticRc::split_array::<1, 100>(registry);
    let mut connections: Vec<Connection<'id, '_>, 100> = Vec::uninit();
    for i in 0..100 {
        let registry = &arr[i];
        let _result = connections.push(Connection { registry });
    }
}
struct Registry {}

struct Connection<'id, 'registry> {
    registry: &'registry LCell<'id, Registry>,
}

impl<'id, 'registry> Connection<'id, 'registry> {
    fn a(&mut self, owner: &mut LCellOwner<'id>) {
        let value = self.registry.rw(owner);
    }
}
