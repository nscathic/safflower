#[test]
pub fn load() {
    safflower_macro::load!("test-data/mini-2loc.txt");

    println!("\n\n{}\n\n", safflower_generated::greet(safflower_generated::get_locale()));
}
