use std::{thread::sleep, time::Duration};

use qcell::{LCell, LCellOwner};
use static_rc::StaticRc;

pub struct App<'id, 'node> {
    node: StaticRc<LCell<'id, Node<'id, 'node, i32>>, 1, 3>,
}
pub struct Node<'id, 'node, T> {
    value: T,
    prev: Option<StaticRc<LCell<'id, Node<'id, 'node, T>>, 1, 3>>,
    next: Option<StaticRc<LCell<'id, Node<'id, 'node, T>>, 1, 3>>,
}

impl<T> Drop for Node<'_, '_, T> {
    fn drop(&mut self) {
        println!("NodeDrop");
    }
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
        let head = StaticRc::<_, 3, 3>::new(owner.cell(Node::new(0)));
        return;
        let (head2, head1) = StaticRc::split::<2, 1>(head);
        let (head2, head3) = StaticRc::split::<1, 1>(head2);
        let seconds = StaticRc::<_, 3, 3>::new(owner.cell(Node::new(1)));
        let tail = StaticRc::<_, 3, 3>::new(owner.cell(Node::new(2)));
        let [seconds1, seconds2, seconds3] = StaticRc::split_array::<1, 3>(seconds);
        let [tail1, tail2, tail3] = StaticRc::split_array::<1, 3>(tail);
        head1.rw(&mut owner).prev = Some(tail1);
        head1.rw(&mut owner).next = Some(seconds1);
        seconds2.rw(&mut owner).prev = Some(head1);
        seconds2.rw(&mut owner).next = Some(tail2);
        tail3.rw(&mut owner).prev = Some(seconds3);
        tail3.rw(&mut owner).next = Some(head2);
        let app = App { node: head3 };
        sleep(Duration::from_secs(4));
    });
}

fn qwer<'id>(owner: &mut LCellOwner<'id>, head: &LCell<'id, Node<i32>>) {
    let head = owner.rw(head);
    head.next = None;
}

fn app<'id, 'node>(owner: &mut LCellOwner<'id>, app: App<'id, 'node>) {
    let node = app.node.rw(owner);
    if let Some(prev) = &node.prev {}
    if let Some(next) = &node.next {
        println!("START");
        node.next = None;
        println!("END");
    }
}
