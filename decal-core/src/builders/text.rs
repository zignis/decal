use crate::{
    attributes::IntoPaintStack,
    capabilities::*,
    layout::{
        Node,
        NodeKind,
        StencilScope,
        StencilType,
        TextMeta,
        Typography,
    },
    macros::impl_node_builder,
    paint::{
        Appearance,
        IntoResources,
        Resource,
    },
    primitives::{
        Ellipsize,
        FontStyle,
        FontWeight,
        Paint,
    },
};
use taffy::prelude::*;

/// Text node.
#[derive(Debug, Default)]
pub struct Text {
    meta: TextMeta,
    layout: Style,
    visual: Appearance,
    typography: Typography,
    resources: Vec<Resource>,
}

impl_node_builder!(
    Text,
    build(this) {
        let mut meta = this.meta;
        meta.typography(this.typography.clone());

        Node::new(
            NodeKind::Text(Box::new(meta)),
            this.layout,
            this.visual,
            Some(this.typography),
            this.resources
        )
    }
);

impl Text {
    /// Creates a new text node from the provided content.
    ///
    /// # Arguments
    /// - `content`: The text content convertible into one or more [`TextSpan`]
    ///   values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let scene = decal! {
    ///     Column {
    ///         Text("another example text")
    ///         Text(text! {
    ///             "some ",
    ///             ("example ", { weight: FontWeight::Bold }),
    ///             "text."
    ///         })
    ///     }
    /// };
    /// ```
    ///
    /// See the [`text!`] macro for more advanced span composition examples.
    ///
    /// # Returns
    /// - [`Self`]
    ///
    /// [`text!`]: decal_macros::text
    pub fn new<S>(content: S) -> Self
    where
        S: IntoText,
    {
        let spans = content.into_text_spans();
        let mut resources = Vec::new();

        for span in &spans {
            resources.extend(span.resources.clone());
        }

        Self {
            meta: TextMeta::new(spans),
            resources,
            ..Default::default()
        }
    }

    /// Use this text as a stencil mask to the provided source (paint stack).
    ///
    /// # Arguments
    /// - `value`: The paint stack used as a stencil source, convertible via
    ///   [`IntoPaintStack`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let gradient = LinearGradient::right().stops([(0.0, rgb(0xff0000)), (1.0, rgb(0x00ff00))]);
    ///
    /// let scene = decal! {
    ///     Text("hello")
    ///         .stencil(gradient)
    /// };
    /// ```
    ///
    /// # Returns
    /// - [`Self`]
    pub fn stencil<T>(mut self, value: T) -> Self
    where
        T: IntoPaintStack,
    {
        let paint = value.into_paint_stack();
        self.meta.stencil_paint(paint.clone());
        self.add_resources(paint);
        self
    }

    /// Sets the scope that determines which glyphs participate in stencil mask
    /// generation.
    ///
    /// # Arguments
    /// - `scope`: The [`StencilScope`] defining which glyph categories are
    ///   included in the stencil mask.
    ///
    /// # Examples
    ///
    /// Exclude bitmap glyphs such as emojis from the stencil mask while still
    /// rendering them normally.
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let gradient = LinearGradient::right().stops([(0.0, rgb(0xff0000)), (1.0, rgb(0x00ff00))]);
    ///
    /// let scene = decal! {
    ///     Text("hello 🐠")
    ///         .stencil(gradient)
    ///         .stencil_scope(StencilScope::VectorGlyphs)
    /// };
    /// ```
    ///
    /// Include all glyphs including bitmap glyphs and emojis in the stencil
    /// mask.
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let gradient = LinearGradient::right().stops([(0.0, rgb(0xff0000)), (1.0, rgb(0x00ff00))]);
    ///
    /// let scene = decal! {
    ///     Text("hello 🐠")
    ///         .stencil(gradient)
    ///         .stencil_scope(StencilScope::AllGlyphs)
    /// };
    /// ```
    ///
    /// # Returns
    /// - [`Self`]
    pub fn stencil_scope(mut self, scope: StencilScope) -> Self {
        self.meta.stencil_scope(scope);
        self
    }

    /// Sets how the stencil mask is derived from the rendered glyph content.
    ///
    /// This determines which channel of the glyph rendering is sampled to
    /// construct the stencil mask, which directly influences how strongly paint
    /// effects are applied across the text.
    ///
    /// # Arguments
    /// - `value`: The [`StencilType`] that defines whether alpha or luminance
    ///   data is used for stencil generation.
    ///
    /// # Examples
    ///
    /// Use glyph alpha values to control stencil intensity.
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let gradient = LinearGradient::right().stops([(0.0, rgb(0xff0000)), (1.0, rgb(0x00ff00))]);
    ///
    /// let scene = decal! {
    ///     Text("hello 🐠")
    ///         .stencil(gradient)
    ///         .stencil_type(StencilType::Alpha)
    /// };
    /// ```
    ///
    /// Use glyph luminance values to control stencil intensity based on
    /// brightness.
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let gradient = LinearGradient::right().stops([(0.0, rgb(0x0000ff)), (1.0, rgb(0xffff00))]);
    ///
    /// let scene = decal! {
    ///     Text("hello 🐠")
    ///         .stencil(gradient)
    ///         .stencil_type(StencilType::Luminance)
    /// };
    /// ```
    ///
    /// # Returns
    /// - [`Self`]
    pub fn stencil_type(mut self, value: StencilType) -> Self {
        self.meta.stencil_type(value);
        self
    }
}

impl Hideable for Text {
    fn hidden(mut self, value: bool) -> Self {
        self.layout.display = if value { Display::None } else { Display::Block };
        self
    }
}

impl Background for Text {}
impl Dimensions for Text {}
impl Margin for Text {}
impl Opacity for Text {}
impl Positioned for Text {}
impl Transformation for Text {}
impl Textual for Text {}
impl SelfAlignment for Text {}
impl Visibility for Text {}
impl FilterEffects for Text {}
impl Blendable for Text {}

//

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub(crate) content: String,
    pub(crate) typography: Typography,
    pub(crate) resources: Vec<Resource>,
    pub(crate) hidden: bool,
}

impl TextSpan {
    /// Creates a new text span.
    ///
    /// Manual construction of [`TextSpan`] is supported, but the [`text!`]
    /// macro is the preferred way to build styled text spans in most cases.
    /// See [`Text::new`] for typical usage patterns.
    ///
    /// # Arguments
    /// - `content`: The content of the span.
    ///
    /// # Returns
    /// - [`Self`]
    ///
    /// [`text!`]: decal_macros::text
    pub fn new(content: String) -> Self {
        Self {
            content,
            typography: Default::default(),
            resources: Vec::new(),
            hidden: false,
        }
    }

    /// Sets the font family for the text span.
    ///
    /// # Arguments
    /// - `family`: The font family name.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn family<T>(mut self, family: T) -> Self
    where
        T: Into<Option<String>>,
    {
        self.typography.family = family.into();
        self
    }

    /// Sets the font size for the text span.
    ///
    /// # Arguments
    /// - `size`: The font size.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn size<T>(mut self, size: T) -> Self
    where
        T: Into<Option<f32>>,
    {
        self.typography.size = size.into();
        self
    }

    /// Sets the line height for the text span.
    ///
    /// # Arguments
    /// - `line_height`: The line height.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn line_height<T>(mut self, line_height: T) -> Self
    where
        T: Into<Option<f32>>,
    {
        self.typography.line_height = line_height.into();
        self
    }

    /// Sets the font weight for the text span.
    ///
    /// # Arguments
    /// - `weight`: The [`FontWeight`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn weight<T>(mut self, weight: T) -> Self
    where
        T: Into<Option<FontWeight>>,
    {
        self.typography.weight = weight.into();
        self
    }

    /// Sets the text color for the span.
    ///
    /// # Arguments
    /// - `color`: The [`Paint`] value applied to the text.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn color<T>(mut self, color: T) -> Self
    where
        T: Into<Option<Paint>>,
    {
        let color = color.into();
        self.typography.color = color.clone();

        if let Some(resources) = color.map(|c| c.into_resources()) {
            self.resources.extend(resources);
        }

        self
    }

    /// Sets the font style for the text span.
    ///
    /// # Arguments
    /// - `style`: The [`FontStyle`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn style<T>(mut self, style: T) -> Self
    where
        T: Into<Option<FontStyle>>,
    {
        self.typography.style = style.into();
        self
    }

    /// Sets the letter spacing for the text span.
    ///
    /// # Arguments
    /// - `letter_spacing`: The letter spacing.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn letter_spacing<T>(mut self, letter_spacing: T) -> Self
    where
        T: Into<Option<f32>>,
    {
        self.typography.letter_spacing = letter_spacing.into();
        self
    }

    /// Sets the ellipsis behavior for the text span.
    ///
    /// # Arguments
    /// - `value`: The [`Ellipsize`] behavior to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn ellipsize<T>(mut self, value: T) -> Self
    where
        T: Into<Option<Ellipsize>>,
    {
        self.typography.ellipsize = value.into();
        self
    }

    /// Marks the text span as hidden or visible.
    ///
    /// # Arguments
    /// - `value`: Whether the span should opt out of rendering.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn hidden(mut self, value: bool) -> Self {
        self.hidden = value;
        self
    }
}

/// Conversion trait for values that can be interpreted as a text span.
pub trait IntoTextSpan {
    /// Converts the value into a [`TextSpan`].
    fn into_text_span(self) -> TextSpan;
}

/// Converts a string slice into a text span.
impl IntoTextSpan for &str {
    fn into_text_span(self) -> TextSpan {
        TextSpan::new(self.to_string())
    }
}

/// Converts an owned string into a text span.
impl IntoTextSpan for String {
    fn into_text_span(self) -> TextSpan {
        TextSpan::new(self)
    }
}

/// Identity conversion for an existing text span.
impl IntoTextSpan for TextSpan {
    fn into_text_span(self) -> TextSpan {
        self
    }
}

/// Conversion trait for values that can be expanded into multiple text spans.
pub trait IntoText {
    /// Converts the value into a collection of text spans.
    fn into_text_spans(self) -> Vec<TextSpan>;
}

/// Converts a string slice into a single text span.
impl IntoText for &str {
    fn into_text_spans(self) -> Vec<TextSpan> {
        vec![self.into_text_span()]
    }
}

/// Converts an owned string into a single text span.
impl IntoText for String {
    fn into_text_spans(self) -> Vec<TextSpan> {
        vec![self.into_text_span()]
    }
}

/// Converts a collection of span-like values into text spans.
impl<T> IntoText for Vec<T>
where
    T: IntoTextSpan,
{
    fn into_text_spans(self) -> Vec<TextSpan> {
        self.into_iter().map(|x| x.into_text_span()).collect()
    }
}
