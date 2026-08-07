//! # Decal
//!
//! `decal` is a declarative rendering library that lets you describe scenes
//! using a Rust-native DSL and render them to SVG or PNG.
//!
//! Scene descriptions are backend-agnostic and can be serialized into vector
//! (SVG, native) or raster (PNG using [`resvg`]) output while preserving
//! layout semantics and visual fidelity.
//!
//! ## Usage
//!
//! At a high level, the flow looks like this:
//!
//! 1. Build a scene using the `decal!` macro and primitives like `Row`, `Column`, `Text`, `Image`, etc.
//! 2. Initialize an `Engine` with fonts and options.
//! 3. Render the scene to SVG (`vectorize`) or rasterize it to a bitmap (`rasterize`).
//!
//! ```rust
//! use decal::prelude::*;
//! use std::fs;
//!
//! fn main() {
//!     let mut engine = Engine::new(EngineOptions::default());
//!
//!     let mut scene = decal! {
//!         Block {}
//!             .size(256)
//!             .background(rgb(0xffffff))
//!     };
//!
//!     let (svg, _size) = engine
//!         .vectorize(&mut scene, &VectorizeOptions::default())
//!         .unwrap();
//!
//!     fs::write("./markup.svg", svg).unwrap();
//!
//!     let (pixmap, _size) = engine
//!         .rasterize(&mut scene, &RasterizeOptions::default())
//!         .unwrap();
//!
//!     pixmap.save_png("./render.png").unwrap();
//! }
//! ```
//!
//! Explore more [examples](https://github.com/zignis/decal/tree/main/examples).
//!
//! Checkout the project repository for more details: https://github.com/zignis/decal

#[forbid(unsafe_code)]
#[warn(missing_docs)]
//
// Private
mod macros;
mod paint;
#[cfg(test)]
mod test_utils;
mod utils;
//
// Public
pub mod attributes;
pub mod builders;
pub mod capabilities;
pub mod filters;
pub mod layout;
pub mod prelude;
pub mod primitives;
//
// Public macros re-export
pub use decal_macros::*;
