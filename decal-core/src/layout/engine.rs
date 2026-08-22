#[cfg(any(target_arch = "wasm32", feature = "network-async"))]
use crate::layout::fetch_image_async;
#[cfg(all(not(target_arch = "wasm32"), feature = "network"))]
use crate::layout::fetch_image_sync;
use crate::{
    layout::{
        ImageOptions,
        RasterizeError,
        RasterizeOptions,
        Scene,
        VectorizeError,
        VectorizeOptions,
        font::FontRegistry,
    },
    primitives::Size,
};
#[cfg(any(target_arch = "wasm32", feature = "network-async"))]
use futures::future::join_all;
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
    /// # Note for WASM usage
    /// On WASM targets, this synchronous method cannot perform network
    /// requests. If your scene references external images, they will fail to
    /// load. You must use [`rasterize_async`] instead, or provide a custom
    /// `href_string_resolver` in [`ImageOptions`].
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
    ///
    /// [`rasterize_async`]: Engine::rasterize_async
    pub fn rasterize(
        &mut self,
        scene: &mut Scene,
        options: &RasterizeOptions,
    ) -> Result<(Pixmap, Size<f32>), RasterizeError> {
        #[cfg(all(not(target_arch = "wasm32"), feature = "network"))]
        self.preload_images(scene, &options.image);
        let output = self.prepare(scene).rasterize(&self.image_cache, options);
        self.cleanup_cache(&options.image);
        output
    }

    /// Rasterizes the given scene into a [`Pixmap`] asynchronously.
    ///
    /// The primary advantage of this method over [`rasterize`] is that it
    /// asynchronously preloads all external images present in the scene before
    /// rendering the scene.
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
    ///
    /// [`rasterize`]: Engine::rasterize
    #[cfg(any(target_arch = "wasm32", feature = "network-async"))]
    pub async fn rasterize_async(
        &mut self,
        scene: &mut Scene,
        options: &RasterizeOptions<'_>,
    ) -> Result<(Pixmap, Size<f32>), RasterizeError> {
        self.preload_images_async(scene, &options.image).await;
        let output = self.prepare(scene).rasterize(&self.image_cache, options);
        self.cleanup_cache(&options.image);
        output
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

    /// Preloads all external images refs in the scene into the cache
    /// synchronously.
    ///
    /// Used on bare-metal with `network` feature.
    #[cfg(all(not(target_arch = "wasm32"), feature = "network"))]
    fn preload_images(&self, scene: &Scene, opts: &ImageOptions) {
        let refs = scene.image_refs();

        let uncached = {
            let mut cache = self.image_cache.lock();
            let mut uncached = Vec::new();

            if let Some(cap) = NonZeroUsize::new(refs.len())
                && cap > cache.cap()
            {
                cache.resize(cap);
            }

            for href in refs {
                // use cache.get here to promote required images to MRU positions so they aren't
                // accidentally evicted when fetching new images
                if Self::must_refetch_image(href, opts) || cache.get(href).is_none() {
                    uncached.push(href);
                }
            }

            uncached
        };

        let mut fetched = Vec::with_capacity(uncached.len());

        for href in uncached {
            if let Some(image) = fetch_image_sync(href) {
                fetched.push((href.clone(), image));
            }
        }

        let mut cache = self.image_cache.lock();

        for (href, image) in fetched {
            cache.push(href, image);
        }
    }

    /// Preloads all external images refs in the scene into the cache
    /// asynchronously.
    ///
    /// Used on WASM targets and on bare-metal with `network-async` feature.
    #[cfg(any(target_arch = "wasm32", feature = "network-async"))]
    async fn preload_images_async(&self, scene: &Scene, opts: &ImageOptions<'_>) {
        let refs = scene.image_refs();

        let uncached = {
            let mut cache = self.image_cache.lock();
            let mut uncached = Vec::new();

            if let Some(cap) = NonZeroUsize::new(refs.len())
                && cap > cache.cap()
            {
                cache.resize(cap);
            }

            for href in refs {
                // use cache.get here to promote required images to MRU positions so they aren't
                // accidentally evicted when fetching new images
                if Self::must_refetch_image(href, opts) || cache.get(href).is_none() {
                    uncached.push(href);
                }
            }

            uncached
        };

        let futures = uncached
            .into_iter()
            .map(|href| {
                let href = href.clone();

                async move {
                    let image = fetch_image_async(&href).await;
                    (href, image)
                }
            })
            .collect::<Vec<_>>();

        let images = join_all(futures).await;
        let mut cache = self.image_cache.lock();

        for (href, image) in images {
            if let Some(kind) = image {
                cache.push(href, kind);
            }
        }
    }

    /// Returns `true` if the image referenced by `href` must be freshly
    /// fetched.
    fn must_refetch_image(href: &str, opts: &ImageOptions<'_>) -> bool {
        opts.disable_caching
            || opts.cache_ignore_list.iter().any(|item| item == href)
            || opts.cache_ignore_fn.as_ref().is_some_and(|f| f(href))
    }

    /// Cleans up the image cache post rasterization.
    fn cleanup_cache(&self, opts: &ImageOptions<'_>) {
        if opts.disable_caching {
            self.image_cache.lock().clear();
            return;
        }

        let mut cache = self.image_cache.lock();

        for href in &opts.cache_ignore_list {
            cache.pop(href);
        }

        if let Some(ignore_fn) = &opts.cache_ignore_fn {
            let stale = cache
                .iter()
                .map(|(key, _)| key.clone())
                .filter(|key| ignore_fn(key))
                .collect::<Vec<_>>();

            for image in stale {
                cache.pop(&image);
            }
        }
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
