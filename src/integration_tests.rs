#[test]
pub fn absolute_minimal() {
    safflower_macro::load!("test-data/abs_min.txt");
    assert_eq!(safflower_macro::text!(a), "c");
}

#[test]
pub fn minimal() {
    safflower_macro::load!("test-data/greet_en_se.txt");
    assert_eq!(safflower_macro::text!(greet), "Hi!");
}
