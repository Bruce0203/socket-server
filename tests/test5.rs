use qcell::{LCell, LCellOwner};

struct Token(usize);
fn asdf() {
    LCellOwner::scope(|owner| {
        let cell = owner.cell(Token(0));
        let cell = owner.cell(Token(0));
        qwer(&cell);
        qwer(&cell);
        qwer(&cell);
        qwer(&cell);
        qwer(&cell);
        qwer(&cell);
        asd(Stream { socket: &cell });
        asd(Stream { socket: &cell });
        asd(Stream { socket: &cell });
        asd(Stream { socket: &cell });
    });
}

struct Stream<'id, 'a> {
    socket: &'a LCell<'id, Token>,
}

fn qwer<'a, 'b: 'a>(value: &'a LCell<'b, Token>) {
    asd(Stream { socket: value });
    asd(Stream { socket: value });
}
fn asd(stream: Stream) {}
