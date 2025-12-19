mod inner {
    use pub_if::pub_if;

    #[pub_if(feature = "foo")]
    pub struct Struct<T> {
        private_field: T,
        pub public_field: String,
    }

    impl<T> Struct<T> {
        pub fn new(value: T) -> Self {
            Self {
                private_field: value,
                public_field: "hello".to_string(),
            }
        }
    }
}

fn main() {
    use inner::Struct;

    let s = Struct::new(42);
    // public_field is accessible
    let _ = &s.public_field;
    // but private_field should NOT be accessible
    let _ = s.private_field;
    //~^ ERROR: field `private_field` of struct `Struct` is private
}
