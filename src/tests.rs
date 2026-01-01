use core::f32;

use safflower_macro::{load, text};

#[test]
fn absolute_minimal() {
    load!("test-data/abs_min.txt");
    assert_eq!(text!(a), "c");
}

#[test]
fn two_locales_direct() {
    load!("test-data/greet_en_se.txt");

    let locale = localisation::Locale::En;
    assert_eq!(localisation::greet(locale), "Hi!");

    let locale = localisation::Locale::Se;
    assert_eq!(localisation::greet(locale), "Hej!");
}

#[test]
fn two_tight() {
    load!("test-data/tight_two.txt");

    let locale = localisation::Locale::L1;
    assert_eq!(localisation::k1(locale), "11");
    assert_eq!(localisation::k2(locale), "21");

    let locale = localisation::Locale::L2;
    assert_eq!(localisation::k1(locale), "12");
    assert_eq!(localisation::k2(locale), "22");
}

#[test]
fn two_locales_macro() {
    load!("test-data/greet_en_se.txt");
    localisation::set_locale(localisation::Locale::En);
    assert_eq!(text!(greet), "Hi!");

    localisation::set_locale(localisation::Locale::Se);
    assert_eq!(text!(greet), "Hej!");
}

#[test]
fn arg_str() {
    load!("test-data/greet_name_en_it.txt");
    let name = "Tester";

    assert_eq!(text!(greet, name), "Hi Tester!");
}

#[test]
fn arg_string() {
    load!("test-data/greet_name_en_it.txt");
    let name = String::from("Tester");

    assert_eq!(text!(greet, name), "Hi Tester!");
}

#[test]
fn arg_i32() {
    load!("test-data/greet_name_en_it.txt");
    let name = 93393;

    assert_eq!(text!(greet, name), "Hi 93393!");
}

#[test]
fn arg_bool() {
    load!("test-data/greet_name_en_it.txt");
    let name = true;

    assert_eq!(text!(greet, name), "Hi true!");
}

#[test]
fn arg_f32formatter() {
    load!("test-data/float_format.txt");
    let name = f32::consts::E;

    let locale = localisation::Locale::D1;
    assert_eq!(localisation::digits(locale, name), "value: 2.7");
    let locale = localisation::Locale::D2;
    assert_eq!(localisation::digits(locale, name), "value: 2.72");
    let locale = localisation::Locale::D3;
    assert_eq!(localisation::digits(locale, name), "value: 2.718");
}

#[test]
fn separate_entries() {
    load!("test-data/separate_entries.txt");
    assert_eq!(text!(key1), "A");
    assert_eq!(text!(key2), "A");
    localisation::set_locale(localisation::Locale::B);
    assert_eq!(text!(key1), "B");
    assert_eq!(text!(key2), "B");
}

#[test]
fn separate_files() {
    load!("test-data/separate_files.txt");
    assert_eq!(text!(key1), "A");
    assert_eq!(text!(key2), "A");
    localisation::set_locale(localisation::Locale::B);
    assert_eq!(text!(key1), "B");
    assert_eq!(text!(key2), "B");
}

#[test]
fn separate_files_and_locales() {
    load!("test-data/separate_files_wl.txt");
    assert_eq!(text!(key1), "A");
    assert_eq!(text!(key2), "A");
    localisation::set_locale(localisation::Locale::B);
    assert_eq!(text!(key1), "B");
    assert_eq!(text!(key2), "B");
}
