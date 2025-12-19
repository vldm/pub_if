mod inner {
    use pub_if::pub_if;

    #[pub_if(feature = "foo")]
    pub struct Struct {
        private_field: i32,
        pub public_field: String,
    }

    impl Struct {
        pub fn new() -> Self {
            Self {
                private_field: 42,
                public_field: "hello".to_string(),
            }
        }
    }
}

fn main() {
    use inner::Struct;

    #[cfg(not(feature = "foo"))]
    {
        let s = Struct::new();
        // Without feature, only originally pub fields should be accessible
        let _ = &s.public_field;
    }
}
