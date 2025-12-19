#[test]
#[cfg(not(feature = "foo"))]
fn compile_tests_without_feature() {
    let t = trybuild::TestCases::new();

    // These should compile successfully
    t.pass("tests/compile_pass/without_feature_pub_only.rs");

    // These should fail to compile (private fields not accessible)
    t.compile_fail("tests/compile_fail/*.rs");
}

#[test]
#[cfg(feature = "foo")]
fn compile_tests_with_feature() {
    let t = trybuild::TestCases::new();

    // With feature enabled, accessing private fields should work
    t.pass("tests/compile_pass/with_feature_all_public.rs");
}
