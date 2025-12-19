mod inner {
    use pub_if::pub_if;

    #[pub_if(feature = "foo")]
    pub struct Struct<F, B> {
        field: F,
        bar: B,
    }

    impl<F, B> Struct<F, B> {
        pub fn new(field: F, bar: B) -> Self {
            Self { field, bar }
        }
    }
}

fn main() {
    use inner::Struct;

    #[cfg(feature = "foo")]
    {
        let s = Struct::new(42, "hello");
        // With feature enabled, all fields should be public
        let _ = s.field;
        let _ = s.bar;
    }
}
