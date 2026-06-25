//! # Decal
//!
//! `decal` is a declarative rendering library that lets you describe scenes
//! using a Rust-native DSL and render them to SVG or PNG.
//!
//! Scene descriptions are backend-agnostic and can be serialized into vector
//! (SVG, native) or raster (PNG using [`resvg`]) output while preserving
//! layout semantics and visual fidelity.
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
