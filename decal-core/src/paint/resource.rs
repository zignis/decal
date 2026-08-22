use crate::{
    filters::Filter,
    primitives::{
        ClipPath,
        ImagePaint,
        LinearGradient,
        Mask,
        Paint,
        PaintInner,
        Path,
        Pattern,
        RadialGradient,
    },
    utils::IsDefault,
};
use enum_display::EnumDisplay;
use std::hash::Hash;

/// The resource that will be emitted inside `<defs>` and needs to be deduped.
#[derive(Debug, Hash, Eq, PartialEq, Clone, EnumDisplay)]
pub(crate) enum Resource {
    #[display("{0}")]
    LinearGradient(LinearGradient),
    #[display("{0}")]
    RadialGradient(RadialGradient),
    #[display("{0}")]
    Pattern(Pattern),
    #[display("{0}")]
    ImagePaint(ImagePaint),
    #[display("{0}")]
    Filter(Filter),
    #[display("{0}")]
    ClipPath(ClipPath),
    #[display("{0}")]
    Mask(Mask),
    #[display("{0}")]
    Path(Path),
}

impl Resource {
    /// Returns the URL of all external image objects present in this resource.
    ///
    /// # Returns
    /// - A vector of external image URLs
    pub(crate) fn image_refs(&self) -> Vec<String> {
        let mut image_refs = Vec::new();

        match self {
            Resource::Filter(filter) => image_refs.extend(filter.image_refs()),
            Resource::ImagePaint(paint) => {
                if let Some(href) = paint.as_href() {
                    image_refs.push(href.to_string());
                }
            }
            _ => {}
        }

        image_refs
    }
}

/// Conversion trait for extracting render resources from higher-level values.
pub(crate) trait IntoResources {
    /// Converts the value into a collection of render resources.
    ///
    /// # Returns
    /// - A vector of [`Resource`] values required for rendering.
    fn into_resources(self) -> Vec<Resource>;
}

/// Identity conversion for an existing resource collection.
impl IntoResources for Vec<Resource> {
    #[inline]
    fn into_resources(self) -> Vec<Resource> {
        self
    }
}

impl IntoResources for Paint {
    #[inline]
    fn into_resources(self) -> Vec<Resource> {
        match self.0 {
            PaintInner::None | PaintInner::Color(_) => Vec::new(),
            PaintInner::LinearGradient(value) => value.into_resources(),
            PaintInner::RadialGradient(value) => value.into_resources(),
            PaintInner::Pattern(value) => value.into_resources(),
            PaintInner::Image(value) => value.into_resources(),
        }
    }
}

impl IntoResources for Pattern {
    #[inline]
    fn into_resources(self) -> Vec<Resource> {
        vec![Resource::Pattern(self)]
    }
}

impl IntoResources for ImagePaint {
    #[inline]
    fn into_resources(self) -> Vec<Resource> {
        vec![Resource::ImagePaint(self)]
    }
}

impl IntoResources for Filter {
    #[inline]
    fn into_resources(self) -> Vec<Resource> {
        if self.is_default() {
            Vec::new()
        } else {
            vec![Resource::Filter(self)]
        }
    }
}

//

impl From<LinearGradient> for Resource {
    #[inline]
    fn from(value: LinearGradient) -> Self {
        Self::LinearGradient(value)
    }
}

impl From<RadialGradient> for Resource {
    #[inline]
    fn from(value: RadialGradient) -> Self {
        Self::RadialGradient(value)
    }
}

impl From<Pattern> for Resource {
    #[inline]
    fn from(value: Pattern) -> Self {
        Self::Pattern(value)
    }
}

impl From<ImagePaint> for Resource {
    #[inline]
    fn from(value: ImagePaint) -> Self {
        Self::ImagePaint(value)
    }
}

impl From<Filter> for Resource {
    #[inline]
    fn from(value: Filter) -> Self {
        Self::Filter(value)
    }
}

impl From<ClipPath> for Resource {
    #[inline]
    fn from(value: ClipPath) -> Self {
        Self::ClipPath(value)
    }
}

impl From<Mask> for Resource {
    #[inline]
    fn from(value: Mask) -> Self {
        Self::Mask(value)
    }
}

impl From<Path> for Resource {
    #[inline]
    fn from(value: Path) -> Self {
        Self::Path(value)
    }
}
