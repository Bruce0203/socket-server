#[cfg(feature = "nothing")]
mod a {
    use std::marker::PhantomData;

    #[test]
    fn qwer() {
        println!("IH");
        let mut app = App;
        app.join();
        app.join();
        app.join();
    }

    pub struct App<'a>(PhantomData<&'a ()>);
    impl<'a> App<'a> {
        fn join(&'a mut self) {
            self.join2();
            self.join2();
            self.join2();
            self.join2();
            self.join2();
            self.join2();
        }
        fn join2(&'a mut self) {}
        fn join3(&mut self) {}
    }
}
