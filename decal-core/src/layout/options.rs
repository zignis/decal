use std::{
    fmt::{
        Debug,
        Formatter,
    },
    sync::Arc,
};
use tiny_skia::Transform;
use usvg::{
    ImageKind,
    ImageRendering,
    ShapeRendering,
    TextRendering,
};

/// Controls how width and height attributes are emitted on the `<svg>` element.
#[derive(Debug, Clone, Default)]
pub enum SvgDimensions {
    /// Omit explicit `width` and `height` attributes. The SVG will rely solely
    /// on its `viewBox`.
    #[default]
    Omit,
    /// Use the computed layout size, from the root node of the scene, for the
    /// `width` and `height` attributes.
    Layout,
    /// Use custom values for the `width` and `height` attributes.
    Custom {
        /// The value used for the `width` attribute.
        width: String,
        /// The value used for the `height` attribute.
        height: String,
    },
}

/// Options controlling scene vectorization output.
#[derive(Debug, Clone, Default)]
pub struct VectorizeOptions {
    /// Controls how `width` and `height` are emitted on the `<svg>` element.
    pub svg_dimensions: SvgDimensions,
    /// Omits the `xmlns` attribute when set to `true`.
    pub omit_svg_xmlns: bool,
}

/// Resolver function for [Data URL](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs) based image references.
///
/// This function will be called with mime, decoded base64 data, and parsing
/// options.
pub type ImageHrefDataResolver<'a> =
    Arc<dyn Fn(&str, Arc<Vec<u8>>, &usvg::Options) -> Option<ImageKind> + Send + Sync + 'a>;

/// Resolver for string-based image references.
///
/// This function will be called with image href and parsing options.
pub type ImageHrefStringResolver<'a> =
    Arc<dyn Fn(&str, &usvg::Options) -> Option<ImageKind> + Send + Sync + 'a>;

/// Function for filtering out the images to cache.
pub type CacheIgnoreFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Configuration options controlling image loading and caching behavior.
#[derive(Clone, Default)]
pub struct ImageOptions<'a> {
    /// Disables image caching entirely when set to `true`.
    pub disable_caching: bool,
    /// A list of image `href`s that should bypass the cache.
    pub cache_ignore_list: Vec<String>,
    /// A predicate used to determine whether a given `href` should bypass the
    /// cache.
    pub cache_ignore_fn: Option<CacheIgnoreFn>,
    /// Optional resolver for data-based image references.
    pub href_data_resolver: Option<ImageHrefDataResolver<'a>>,
    /// Optional resolver for string-based image references.
    pub href_string_resolver: Option<ImageHrefStringResolver<'a>>,
}

impl<'a> Debug for ImageOptions<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageOptions")
            .field("disable_caching", &self.disable_caching)
            .field("cache_ignore_list", &self.cache_ignore_list)
            .finish()
    }
}

/// Options controlling scene rasterization behavior.
#[derive(Debug, Clone, Default)]
pub struct RasterizeOptions<'a> {
    /// Draws node bounding boxes if set to `true`.
    pub debug: bool,
    /// The options used during the scene vectorization stage.
    pub vectorize_options: VectorizeOptions,
    /// Controls shape rendering quality and hinting.
    pub shape_rendering: ShapeRendering,
    /// Controls text rendering quality and hinting.
    pub text_rendering: TextRendering,
    /// Controls image rendering quality.
    pub image_rendering: ImageRendering,
    /// Transform applied to the scene before rasterization.
    pub root_transform: Transform,
    /// Image loading and caching options.
    pub image: ImageOptions<'a>,
}
