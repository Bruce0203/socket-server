use qcell::LCell;
use static_rc::StaticRc;

#[test]
fn test() {
    let value = Rcs::new(123);
}

enum Rcs<'id, T> {
    V1(StaticRc<LCell<'id, T>, 1, 3>),
    V2(StaticRc<LCell<'id, T>, 2, 3>),
    V3(StaticRc<LCell<'id, T>, 3, 3>),
}

impl<'id, T> Rcs<'id, T> {
    pub fn new(value: T) -> Self {
        Self::V3(StaticRc::new(LCell::new(value)))
    }

    pub fn clone(mut self) -> Self {
        match self {
            Rcs::V1(v) => todo!(),
            Rcs::V2(v) => {
                let (rc1, rc2) = StaticRc::split::<1, 1>(v);
                self = Rcs::V1(rc1);
                Rcs::V1(rc2)
            }
            Rcs::V3(v) => {
                let (rc1, rc2) = StaticRc::split::<2, 1>(v);
                self = Rcs::V2(rc1);
                Rcs::V1(rc2)
            }
        }
    }
}
