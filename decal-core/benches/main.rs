use criterion::{
    criterion_group,
    criterion_main,
};

mod filters;
mod layout;
mod paint;
mod text;
mod utils;

criterion_group!(
    benches,
    text::bench,
    layout::bench,
    paint::bench,
    filters::bench
);

criterion_main!(benches);
