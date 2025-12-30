use safflower::{load, text};
// use safflower_core as core;

fn main() {
    loading_formatting_noargs();
}

#[allow(clippy::useless_format)]
fn loading_formatting_noargs() {
    load!("src/bin/lorem256_1024.txt");
    
    let n = 1_000_000;

    let t0 = std::time::Instant::now();
    for _ in 0..n {
        _ = format!(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nunc \
            eget metus dapibus, hendrerit libero et, posuere massa. Sed eget \
            odio magna. Suspendisse potenti. In id tellus semper enim \
            molestie ornare. Donec semper sapien non luctus lobortis. \
            Praesent est."
        );
    }
    let time_format = t0.elapsed();

    let t0 = std::time::Instant::now();
    for _ in 0..n {
        _ = text!(line0);
    }
    let time_text = t0.elapsed();

    println!(
        "{n} format!s took {:.2} µs\n\
         {n} text!s   took {:.2} µs = {:.1} %",
        time_format.as_secs_f32()*1000.0,
        time_text.as_secs_f32()*1000.0,
        time_text.as_secs_f32() / time_format.as_secs_f32() * 100.
    );
}
