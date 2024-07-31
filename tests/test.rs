use qcell::{LCell, LCellOwner};

pub struct Node<'id, 'node, T> {
    value: T,
    prev: Option<&'node LCell<'id, Node<'id, 'node, T>>>,
    next: Option<&'node LCell<'id, Node<'id, 'node, T>>>,
}

impl<'id, 'node, T> Node<'id, 'node, T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            prev: None,
            next: None,
        }
    }
}

#[test]
fn linked_list_test() {
    LCellOwner::scope(|mut owner| {
        let head = owner.cell(Node::new(0));
        let seconds = owner.cell(Node::new(1));
        let tail = owner.cell(Node::new(2));
        head.rw(&mut owner).prev = Some(&tail);
        head.rw(&mut owner).next = Some(&seconds);
        seconds.rw(&mut owner).prev = Some(&head);
        seconds.rw(&mut owner).next = Some(&tail);
        tail.rw(&mut owner).prev = Some(&seconds);
        tail.rw(&mut owner).next = Some(&head);
        qwer(&mut owner, &head);
    });
}

fn qwer<'id>(owner: &mut LCellOwner<'id>, head: &LCell<'id, Node<i32>>) {
    let head = owner.rw(head);
    head.next = None;
}
