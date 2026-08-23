use crate::utils::setup_engine;
use criterion::Criterion;
use decal::prelude::*;
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut engine = setup_engine();

    let mut scene = decal! {
        Row {
            Column {
                Row {
                    Column {
                        Row {
                            for _ in 0..25 {
                                Column {
                                    for _ in 0..500 {
                                        Block {}.size((100, 100))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    c.bench_function("deep_nesting", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });
}
