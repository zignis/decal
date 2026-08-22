use crate::{
    layout::RenderContext,
    primitives::CrossOrigin,
    utils::ElementWriter,
};
use enum_display::EnumDisplay;
use quick_xml::escape::escape;
use std::fmt::{
    Display,
    Formatter,
    Write,
};
use taffy::Size;

#[derive(Debug, Hash, Eq, PartialEq, Clone, EnumDisplay)]
pub(crate) enum ImageSourceInner {
    /// External image reference.
    #[display("{0}")]
    Href(String),
    /// Inline SVG content rendered verbatim.
    #[display("{0}")]
    Svg(String),
}

/// The source of an image node.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ImageSource(ImageSourceInner);

impl Display for ImageSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//

#[derive(Debug, Clone, Default)]
pub(crate) struct ImageMeta {
    pub(crate) source: ImageSource,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) cross_origin: Option<CrossOrigin>,
}

impl ImageMeta {
    /// Creates a new [`ImageMeta`] instance.
    ///
    /// # Arguments
    /// - `source`: The [`ImageSource`] value.
    /// - `width`: The intrinsic width of the image.
    /// - `height`: The intrinsic height of the image.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn new<S>(source: S, width: f32, height: f32) -> Self
    where
        S: Into<ImageSource>,
    {
        Self {
            source: source.into(),
            width,
            height,
            ..Default::default()
        }
    }

    /// Computes the final size of the image.
    ///
    /// # Note
    /// Aspect ratio is preserved when only a single dimension is constrained.
    ///
    /// # Arguments
    /// - `known_dimensions`: The known layout dimensions provided by the
    ///   `taffy`.
    ///
    /// # Returns
    /// - The resolved image size.
    pub(crate) fn measure(&mut self, known_dimensions: Size<Option<f32>>) -> Size<f32> {
        match (known_dimensions.width, known_dimensions.height) {
            (Some(width), Some(height)) => Size { width, height },
            (Some(width), None) => Size {
                width,
                height: (width / self.width.max(1.0)) * self.height,
            },
            (None, Some(height)) => Size {
                width: (height / self.height.max(1.0)) * self.width,
                height,
            },
            (None, None) => Size {
                width: self.width,
                height: self.height,
            },
        }
    }

    /// Renders the image into the output stream.
    ///
    /// # Arguments
    /// - `ctx`: The current [`RenderContext`].
    /// - `layout`: The computed layout for the image node.
    pub(crate) fn render<W>(
        &self,
        ctx: &mut RenderContext<W>,
        layout: taffy::Layout,
    ) -> std::fmt::Result
    where
        W: Write,
    {
        let Size { width, height } = layout.size;
        match &self.source.inner() {
            ImageSourceInner::Href(href) => ElementWriter::new(ctx.out, "image")?
                .attr("href", href.as_str())?
                .attrs([("width", width), ("height", height)])?
                .attr("crossorigin", self.cross_origin.map(|x| (x,)))?
                .close(),
            ImageSourceInner::Svg(svg) => ctx.out.write_str(svg.as_str()),
        }
    }
}

impl ImageSource {
    /// Creates an [`ImageSource`] referencing an external URL. The URL is
    /// escaped before being embedded in the output.
    ///
    /// # Arguments
    /// - `url`: The image URL.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn href<S>(url: S) -> Self
    where
        S: AsRef<str>,
    {
        Self(ImageSourceInner::Href(escape(url.as_ref()).to_string()))
    }

    /// Creates an [`ImageSource`] from raw SVG markup. The SVG content is
    /// emitted verbatim during rendering.
    ///
    /// # Arguments
    /// - `svg`: The SVG markup.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn svg<S>(svg: S) -> Self
    where
        S: AsRef<str>,
    {
        Self(ImageSourceInner::Svg(svg.as_ref().to_string()))
    }

    /// Returns the underlying image source representation.
    pub(crate) fn inner(&self) -> &ImageSourceInner {
        &self.0
    }

    /// Returns the href if this is an external image reference.
    pub fn as_href(&self) -> Option<&str> {
        match &self.0 {
            ImageSourceInner::Href(href) if !href.is_empty() => Some(href.as_str()),
            _ => None,
        }
    }
}

impl Default for ImageSource {
    fn default() -> Self {
        ImageSource::href("")
    }
}

impl<T> From<T> for ImageSource
where
    T: AsRef<str>,
{
    #[inline]
    fn from(value: T) -> Self {
        ImageSource::href(value.as_ref())
    }
}
