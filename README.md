# Decal

[![Latest Version](https://img.shields.io/crates/v/decal.svg)](https://crates.io/crates/decal)
[![Rust Documentation](https://docs.rs/decal/badge.svg)](https://docs.rs/decal)

A declarative graphics rendering library that lets you describe scenes using a Rust-native DSL and render them to SVG or
PNG.

## Usage

At a high level, the flow looks like this:

1. Build a scene using the `decal!` macro and primitives like `Row`, `Column`, `Text`, `Image`, etc.
2. Initialize an `Engine` with fonts and options.
3. Render the scene to SVG (`vectorize`) or rasterize it to a bitmap (`rasterize`).

```rust
use decal::prelude::*;
use std::fs;

fn main() {
    let mut engine = Engine::new(EngineOptions::default());

    let mut scene = decal! {
        Block {}
            .size(256)
            .background(rgb(0xffffff))
    };

    let (svg, _size) = engine
        .vectorize(&mut scene, &VectorizeOptions::default())
        .unwrap();

    fs::write("./markup.svg", svg).unwrap();

    let (pixmap, _size) = engine
        .rasterize(&mut scene, &RasterizeOptions::default())
        .unwrap();

    pixmap.save_png("./render.png").unwrap();
}
```

Explore [examples](https://github.com/zignis/decal/tree/main/examples).

## License

[MIT](https://github.com/zignis/decal/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/zignis/decal/blob/main/LICENSE-APACHE)
