use crate::utils::{
    get_sample_image,
    setup_engine,
};
use criterion::Criterion;
use decal::prelude::*;
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut engine = setup_engine();

    let mut scene = decal! {
        Row {
            for _ in 0..100 {
                Block {}.size((50, 50)).corner_radius(5).background(rgb(0xff0000))
                Block {}.size((50, 50)).corner_radius(10).background(rgb(0x00ff00))
                Block {}.size((50, 50)).corner_radius(15).background(rgb(0x0000ff))
                Block {}.size((50, 50)).corner_radius(25).background(rgb(0xffff00))
                Block {}.size((50, 50)).corner_radius(30).background(rgb(0x00ffff))
                Block {}.size((50, 50)).corner_radius(35).background(rgb(0xff00ff))
            }
        }.gap(10)
    };

    c.bench_function("clipping_and_radii", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });
}
