use crate::{
    layout::{
        RasterizeError,
        RasterizeOptions,
        Scene,
        VectorizeError,
        VectorizeOptions,
        font::FontRegistry,
    },
    primitives::Size,
};
use lru::LruCache;
use parking_lot::Mutex;
use smart_default::SmartDefault;
use std::{
    fmt::Write,
    num::NonZeroUsize,
    sync::Arc,
};
use tiny_skia::Pixmap;
use usvg::{
    ImageKind,
    Tree,
};

/// Wrapper around the image data (to defer SVG parsing until rasterization).
#[derive(Clone)]
pub(crate) enum CachedImage {
    Resolved(Box<ImageKind>),
    DeferredSvg(Arc<Vec<u8>>),
}

impl CachedImage {
    pub(crate) fn into_image_kind(self, usvg_opts: &usvg::Options) -> Option<ImageKind> {
        match self {
            CachedImage::Resolved(kind) => Some(*kind),
            CachedImage::DeferredSvg(data) => {
                Tree::from_data(&data, usvg_opts).ok().map(ImageKind::SVG)
            }
        }
    }
}

/// Shared image cache type used to deduplicate decoded raster images across
/// renders.
pub(crate) type ImageCache = Arc<Mutex<LruCache<String, CachedImage>>>;

/// Default capacity used for the image cache.
const DEFAULT_IMAGE_CACHE_CAP: NonZeroUsize = NonZeroUsize::new(128).expect("128 is non-zero");

/// Configuration options controlling engine-wide behavior.
#[derive(Debug, SmartDefault)]
pub struct EngineOptions {
    /// Font registry used for text layout and rendering.
    pub fonts: FontRegistry,
    /// Maximum number of images retained in the image cache.
    #[default(DEFAULT_IMAGE_CACHE_CAP)]
    pub image_cache_capacity: NonZeroUsize,
}

/// The rendering engine responsible for preparing scenes and performing
/// vectorization or rasterization.
#[derive(Debug)]
pub struct Engine {
    /// Shared font registry.
    fonts: Arc<Mutex<FontRegistry>>,
    /// Global image cache.
    image_cache: ImageCache,
}

impl Engine {
    /// Creates a new rendering [`Engine`] with the provided options.
    ///
    /// # Arguments
    /// - `options`: The [`EngineOptions`] used to initialize the engine.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new(options: EngineOptions) -> Self {
        Self {
            fonts: Arc::new(Mutex::new(options.fonts)),
            image_cache: Arc::new(Mutex::new(LruCache::new(options.image_cache_capacity))),
        }
    }

    /// Registers a font in the font registry.
    ///
    /// The font becomes available to all subsequent layout and render
    /// operations.
    ///
    /// # Arguments
    /// - `alias`: The font family name used to reference the font.
    /// - `data`: The raw font data.
    pub fn append_font<T>(&mut self, alias: &str, data: T)
    where
        T: Into<Vec<u8>>,
    {
        self.fonts.lock().append_font(alias, data);
    }

    /// Rasterizes the given scene into a [`Pixmap`].
    ///
    /// # Arguments
    /// - `scene`: The scene to rasterize.
    /// - `options`: The [`RasterizeOptions`] value.
    ///
    /// # Returns
    /// - On success, a tuple containing:
    ///     - [`Pixmap`]: Pixmap containing the image data.
    ///     - [`Size<f32>`]: Scene size.
    /// - [`RasterizeError`] on failure.
    pub fn rasterize(
        &mut self,
        scene: &mut Scene,
        options: &RasterizeOptions,
    ) -> Result<(Pixmap, Size<f32>), RasterizeError> {
        self.prepare(scene).rasterize(&self.image_cache, options)
    }

    /// Vectorizes the given scene into an SVG string.
    ///
    /// # Arguments
    /// - `scene`: The scene to vectorize.
    /// - `options`: The [`VectorizeOptions`] value.
    ///
    /// # Returns
    /// - On success, a tuple containing:
    ///     - [`String`]: The SVG string.
    ///     - [`Size<f32>`]: Scene size.
    /// - [`VectorizeError`] on failure.
    pub fn vectorize(
        &mut self,
        scene: &mut Scene,
        options: &VectorizeOptions,
    ) -> Result<(String, Size<f32>), VectorizeError> {
        self.prepare(scene).vectorize(options)
    }

    /// Streams the vectorized SVG representation of the scene to the provided
    /// destination.
    ///
    /// # Arguments
    /// - `destination`: The output writer.
    /// - `scene`: The scene to vectorize.
    /// - `options`: The [`VectorizeOptions`] value.
    ///
    /// # Returns
    /// - Scene size on success.
    /// - [`VectorizeError`] on failure.
    pub fn stream_vector<T>(
        &mut self,
        destination: &mut T,
        scene: &mut Scene,
        options: &VectorizeOptions,
    ) -> Result<Size<f32>, VectorizeError>
    where
        T: Write,
    {
        self.prepare(scene).stream_vector(destination, options)
    }

    /// Prepares the scene for rendering.
    ///
    /// This injects the engine font registry and computes layout.
    fn prepare<'a>(&self, scene: &'a mut Scene) -> &'a mut Scene {
        scene.set_fonts(self.fonts.clone());
        scene.compute_layout();
        scene
    }
}
