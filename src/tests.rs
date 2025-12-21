use core::f32;

use safflower_macro::{load, text};

#[test]
pub fn absolute_minimal() {
    load!("test-data/abs_min.txt");
    assert_eq!(text!(a), "c");
}

#[test]
pub fn two_locales_direct() {
    load!("test-data/greet_en_se.txt");

    let locale = localisation::Locale::En;
    assert_eq!(localisation::greet(locale), "Hi!");

    let locale = localisation::Locale::Se;
    assert_eq!(localisation::greet(locale), "Hej!");
}

#[test]
pub fn two_locales_macro() {
    load!("test-data/greet_en_se.txt");
    localisation::set_locale(localisation::Locale::En);
    assert_eq!(text!(greet), "Hi!");

    localisation::set_locale(localisation::Locale::Se);
    assert_eq!(text!(greet), "Hej!");
}

#[test]
pub fn arg_str() {
    load!("test-data/greet_name_en_it.txt");
    let name = "Tester";

    assert_eq!(text!(greet, name), "Hi Tester!");
}

#[test]
pub fn arg_string() {
    load!("test-data/greet_name_en_it.txt");
    let name = String::from("Tester");

    assert_eq!(text!(greet, name), "Hi Tester!");
}

#[test]
pub fn arg_i32() {
    load!("test-data/greet_name_en_it.txt");
    let name = 93393;

    assert_eq!(text!(greet, name), "Hi 93393!");
}

#[test]
pub fn arg_bool() {
    load!("test-data/greet_name_en_it.txt");
    let name = true;

    assert_eq!(text!(greet, name), "Hi true!");
}

#[test]
pub fn arg_f32formatter() {
    load!("test-data/float_format.txt");
    let name = f32::consts::E;

    let locale = localisation::Locale::D1;
    assert_eq!(localisation::digits(locale, name), "value: 2.7");
    let locale = localisation::Locale::D2;
    assert_eq!(localisation::digits(locale, name), "value: 2.72");
    let locale = localisation::Locale::D3;
    assert_eq!(localisation::digits(locale, name), "value: 2.718");
}
