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
        Block {
            Row {
                Block {}.size((100, 100)).background(rgb(0xff0000)).fx(Filter::blur(5.0))
                Block {}.size((100, 100)).background(rgb(0x00ff00)).fx([Filter::blur(2.0), Filter::contrast(1.5)])
                Block {}.size((100, 100)).background(rgb(0x0000ff)).fx([Filter::sepia(0.8), Filter::hue_rotate(90.0), Filter::blur(1.0)])
                Image(get_sample_image(), 100.0, 100.0).fx([Filter::grayscale(1.0), Filter::invert(0.5)])
            }
                .background(rgb(0x000))
                .fx(
                    Filter::new(|ctx| {
                        ctx.composite()
                            .input(
                                ctx.specular_lighting(LightSource::distant_light(225.0, 35.0))
                                    .input(
                                        ctx.turbulence()
                                            .fractal_noise()
                                            .base_freq(0.1)
                                            .num_octaves(4)
                                            .finish(),
                                    )
                                    .surface_scale(5.0)
                                    .specular_exponent(20.0)
                                    .lighting_color(rgb((255, 240, 200)))
                                    .finish(),
                            )
                            .input2(FilterInput::source_alpha())
                            .operator(CompositeOperator::r#in())
                            .finish();
                    })
                )
        }
            .size((400, 400))
            .background(rgb(0x336699))
            .fx(Filter::new(|ctx| {
                ctx.displacement_map()
                    .input(FilterInput::source_graphic())
                    .map(
                        ctx.turbulence()
                            .base_freq((0.05, 0.05))
                            .num_octaves(3)
                            .fractal_noise()
                            .finish()
                    )
                    .scale(20.0)
                    .x_channel(ChannelSelector::R)
                    .y_channel(ChannelSelector::G)
                    .finish();
            }))
    };

    c.bench_function("filters", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });
}
