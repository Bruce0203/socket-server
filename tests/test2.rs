use ghost_cell::{GhostCell, GhostToken};

pub struct Node<'id, 'node, T> {
    value: T,
    prev: Option<&'node GhostCell<'id, Node<'id, 'node, T>>>,
    next: Option<&'node GhostCell<'id, Node<'id, 'node, T>>>,
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
    GhostToken::new(|mut owner| {
        let head = GhostCell::new(Node::new(0));
        let seconds = GhostCell::new(Node::new(1));
        let tail = GhostCell::new(Node::new(2));
        head.borrow_mut(&mut owner).prev = Some(&tail);
        head.borrow_mut(&mut owner).next = Some(&seconds);
        seconds.borrow_mut(&mut owner).prev = Some(&head);
        seconds.borrow_mut(&mut owner).next = Some(&tail);
        tail.borrow_mut(&mut owner).prev = Some(&seconds);
        tail.borrow_mut(&mut owner).next = Some(&head);
        qwer(&mut owner, &head);
    });
}

fn qwer<'id>(owner: &mut GhostToken<'id>, head: &GhostCell<'id, Node<i32>>) {
    let head = head.borrow_mut(owner);
    head.next = None;
}
