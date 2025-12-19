mod inner {
    use pub_if::pub_if;

    #[pub_if(feature = "foo")]
    pub struct Struct {
        field: i32,
        bar: String,
    }

    impl Struct {
        pub fn new() -> Self {
            Self {
                field: 42,
                bar: "hello".to_string(),
            }
        }
    }
}

fn main() {
    use inner::Struct;

    let s = Struct::new();
    // Without feature enabled, private fields should NOT be accessible
    let _ = s.field;
    //~^ ERROR: field `field` of struct `Struct` is private
}
