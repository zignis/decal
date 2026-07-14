use crate::{
    layout::{
        ImageSource,
        RenderContext,
    },
    macros::nf32,
    paint::{
        IntoResources,
        Resource,
        ResourceIri,
    },
    primitives::{
        BlendMode,
        Color,
        CrossOrigin,
        Length,
        LinearGradient,
        Path,
        Pattern,
        PatternContentUnits,
        PatternUnits,
        RadialGradient,
    },
    utils::{
        ElementWriter,
        Initialized,
        IsDefault,
    },
};
use smart_default::SmartDefault;
use std::fmt::{
    Display,
    Write,
};
use strict_num::NormalizedF32;

#[derive(Debug, Clone, Default)]
pub(crate) enum PaintInner {
    #[default]
    None,
    Color(Color),
    Image(Pattern),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Pattern(Pattern),
}

/// The paint value.
#[derive(Debug, Clone, Default)]
pub struct Paint(pub(crate) PaintInner);

impl Paint {
    /// Creates a new [`Paint`] value representing no paint.
    ///
    /// # Returns
    /// - [`Self`]
    pub const fn none() -> Self {
        Self(PaintInner::None)
    }

    /// Creates a [`Paint`] value from a solid color.
    ///
    /// # Arguments
    /// - `color`: The [`Color`] to use.
    ///
    /// # Returns
    /// - [`Self`]
    pub const fn color(color: Color) -> Self {
        Self(PaintInner::Color(color))
    }

    /// Creates a [`Paint`] value from an image.
    ///
    /// # Arguments
    /// - `image`: The [`ImagePaint`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn image(image: ImagePaint) -> Self {
        image
            .into_pattern()
            .map(|x| Self(PaintInner::Image(x)))
            .unwrap_or(Self::none())
    }

    /// Creates a [`Paint`] value from a [`LinearGradient`].
    ///
    /// # Arguments
    /// - `linear_gradient`: The [`LinearGradient`] to use.
    ///
    /// # Returns
    /// - [`Self`]
    pub const fn linear_gradient(linear_gradient: LinearGradient) -> Self {
        Self(PaintInner::LinearGradient(linear_gradient))
    }

    /// Creates a [`Paint`] value from a [`RadialGradient`].
    ///
    /// # Arguments
    /// - `radial_gradient`: The [`RadialGradient`] to use.
    ///
    /// # Returns
    /// - [`Self`]
    pub const fn radial_gradient(radial_gradient: RadialGradient) -> Self {
        Self(PaintInner::RadialGradient(radial_gradient))
    }

    /// Creates a [`Paint`] value from a [`Pattern`].
    ///
    /// # Arguments
    /// - `pattern`: The [`Pattern`] to use.
    ///
    /// # Returns
    /// - [`Self`]
    pub const fn pattern(pattern: Pattern) -> Self {
        Self(PaintInner::Pattern(pattern))
    }

    /// Returns `true` if this paint represents the absence of paint.
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Paint(PaintInner::None))
    }
}

impl Display for Paint {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            PaintInner::None => f.write_str("none"),
            PaintInner::Color(color) => color.fmt(f),
            PaintInner::LinearGradient(gradient) => write!(f, "url(#{})", gradient.iri()),
            PaintInner::RadialGradient(gradient) => write!(f, "url(#{})", gradient.iri()),
            PaintInner::Image(pattern) | PaintInner::Pattern(pattern) => {
                write!(f, "url(#{})", pattern.iri())
            }
        }
    }
}

/// The image-based paint configuration.
#[derive(Debug, Clone, SmartDefault)]
pub struct ImagePaint {
    source: ImageSource,
    #[default(nf32!(0.5))]
    x: NormalizedF32,
    #[default(nf32!(0.5))]
    y: NormalizedF32,
    #[default(NormalizedF32::ONE)]
    width: NormalizedF32,
    #[default(NormalizedF32::ONE)]
    height: NormalizedF32,
    cross_origin: Option<CrossOrigin>,
}

impl ImagePaint {
    /// Creates a new [`ImagePaint`] instance.
    ///
    /// # Arguments
    /// - `source`: The [`ImageSource`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new<S>(source: S) -> Self
    where
        S: Into<ImageSource>,
    {
        Self {
            source: source.into(),
            ..Default::default()
        }
    }

    /// Sets the horizontal anchor position of the image.
    ///
    /// # Arguments
    /// - `x`: The horizontal value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn x(mut self, x: f32) -> Self {
        self.x = nf32!(x);
        self
    }

    /// Sets the vertical anchor position of the image.
    ///
    /// # Arguments
    /// - `x`: The horizontal value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn y(mut self, y: f32) -> Self {
        self.y = nf32!(y);
        self
    }

    /// Sets the normalized (relative to the painted node) width of the image.
    ///
    /// # Arguments
    /// - `width`: The width value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn width(mut self, width: f32) -> Self {
        self.width = nf32!(width);
        self
    }

    /// Sets the normalized (relative to the painted node) height of the image.
    ///
    /// # Arguments
    /// - `width`: The height value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn height(mut self, height: f32) -> Self {
        self.height = nf32!(height);
        self
    }

    /// Sets the cross-origin policy for the image.
    ///
    /// # Arguments
    /// - `value`: The [`CrossOrigin`] policy to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn cross_origin<T>(mut self, value: T) -> Self
    where
        T: Into<Option<CrossOrigin>>,
    {
        self.cross_origin = value.into();
        self
    }

    /// Positions the image in the top-left corner of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn top_left(mut self) -> Self {
        self.x = nf32!(0.0);
        self.y = nf32!(0.0);
        self
    }

    /// Positions the image in the top-center of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn top_center(mut self) -> Self {
        self.x = nf32!(0.5);
        self.y = nf32!(0.0);
        self
    }

    /// Positions the image in the top-right corner of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn top_right(mut self) -> Self {
        self.x = nf32!(1.0);
        self.y = nf32!(0.0);
        self
    }

    /// Positions the image in the middle-left of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn middle_left(mut self) -> Self {
        self.x = nf32!(0.0);
        self.y = nf32!(0.5);
        self
    }

    /// Positions the image in the center of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn center(mut self) -> Self {
        self.x = nf32!(0.5);
        self.y = nf32!(0.5);
        self
    }

    /// Positions the image in the middle-right of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn middle_right(mut self) -> Self {
        self.x = nf32!(1.0);
        self.y = nf32!(0.5);
        self
    }

    /// Positions the image in the bottom-left corner of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn bottom_left(mut self) -> Self {
        self.x = nf32!(0.0);
        self.y = nf32!(1.0);
        self
    }

    /// Positions the image in the bottom-center of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn bottom_center(mut self) -> Self {
        self.x = nf32!(0.5);
        self.y = nf32!(1.0);
        self
    }

    /// Positions the image in the bottom-right corner of the painted node.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn bottom_right(mut self) -> Self {
        self.x = nf32!(1.0);
        self.y = nf32!(1.0);
        self
    }

    /// Converts the image configuration into a [`Pattern`] element.
    fn into_pattern(self) -> Option<Pattern> {
        Pattern::build(|out| {
            ElementWriter::new(out, "image")?
                .attr("href", (self.source,))?
                .attr("preserveAspectRatio", "none")?
                .attrs([
                    ("x", (1.0 - self.width.get()) * self.x.get()),
                    ("y", (1.0 - self.height.get()) * self.y.get()),
                    ("width", self.width.get()),
                    ("height", self.height.get()),
                ])?
                .attr("crossorigin", self.cross_origin.map(|x| (x,)))?
                .close()
        })
        .map(|pat| {
            pat.pattern_units(PatternUnits::ObjectBoundingBox)
                .pattern_content_units(PatternContentUnits::ObjectBoundingBox)
                .width(Length::units(1.0))
                .height(Length::units(1.0))
        })
        .ok()
    }
}

//

impl From<Color> for Paint {
    #[inline]
    fn from(value: Color) -> Self {
        Paint::color(value)
    }
}

impl From<LinearGradient> for Paint {
    #[inline]
    fn from(value: LinearGradient) -> Self {
        Paint::linear_gradient(value)
    }
}

impl From<RadialGradient> for Paint {
    #[inline]
    fn from(value: RadialGradient) -> Self {
        Paint::radial_gradient(value)
    }
}

impl From<Pattern> for Paint {
    #[inline]
    fn from(value: Pattern) -> Self {
        Paint::pattern(value)
    }
}

impl From<ImagePaint> for Paint {
    #[inline]
    fn from(value: ImagePaint) -> Self {
        Paint::image(value)
    }
}

/// The paint layer with blending and opacity.
#[derive(Debug, Clone, SmartDefault)]
pub struct PaintLayer {
    pub(crate) paint: Paint,
    pub(crate) blend_mode: BlendMode,
    #[default(NormalizedF32::ONE)]
    pub(crate) opacity: NormalizedF32,
}

impl PaintLayer {
    /// Sets the blending mode for the layer.
    ///
    /// # Arguments
    /// - `blend_mode`: The [`BlendMode`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Sets the opacity for the layer.
    ///
    /// # Arguments
    /// - `opacity`: The opacity value where `0.0` is fully transparent and
    ///   `1.0` is fully opaque.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = nf32!(opacity);
        self
    }

    /// Returns `true` if this layer has no visible paint.
    pub(crate) fn is_none(&self) -> bool {
        self.paint.is_none()
    }
}

impl<T> From<T> for PaintLayer
where
    T: Into<Paint>,
{
    #[inline]
    fn from(value: T) -> Self {
        Self {
            paint: value.into(),
            ..Default::default()
        }
    }
}

/// The stack of paint layers rendered in order.
#[derive(Debug, Clone, Default)]
pub struct PaintStack(Vec<PaintLayer>);

impl PaintStack {
    /// Creates a paint stack from an iterator of paint layers.
    ///
    /// # Arguments
    /// - `layers`: The iterable collection of values convertible into
    ///   [`PaintLayer`].
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn new<I, T>(layers: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<PaintLayer>,
    {
        Self(layers.into_iter().map(Into::into).collect())
    }

    /// Returns `true` if the paint stack produces no visible output.
    pub(crate) fn is_none(&self) -> bool {
        self.0.is_empty() || (self.0.len() == 1 && self.0[0].is_none())
    }

    /// Returns `true` if the paint stack requires isolated blending.
    ///
    /// Isolation is needed when any layer uses a non-default [`BlendMode`],
    /// making sure that blending is evaluated within the stack rather than
    /// against previously rendered content.
    fn needs_isolation(&self) -> bool {
        self.0.iter().any(|x| !x.blend_mode.is_default())
    }

    /// Renders the paint stack into the provided render context.
    ///
    /// # Arguments
    /// - `ctx`: The current [`RenderContext`].
    /// - `draw_single_layer`: The closure used to write path data for a single
    ///   layer.
    /// - `draw_cached_layer`: The closure used to generate reusable path
    ///   geometry.
    /// - `visit_layer`: The visitor invoked for each rendered paint layer.
    /// - `visit_group`: The visitor invoked for the enclosing group when
    ///   multiple layers are present.
    pub(crate) fn render<'a, W, D, S, L, G>(
        &self,
        ctx: &mut RenderContext<W>,
        draw_single_layer: D,
        draw_cached_layer: S,
        visit_layer: L,
        visit_group: G,
    ) -> std::fmt::Result
    where
        W: Write,
        D: FnOnce(&mut W) -> std::fmt::Result,
        S: FnOnce(&mut String) -> std::fmt::Result,
        L: Fn(
            ElementWriter<W, Initialized>,
            bool,
        ) -> Result<ElementWriter<W, Initialized>, std::fmt::Error>,
        G: Fn(
            ElementWriter<W, Initialized>,
        ) -> Result<ElementWriter<W, Initialized>, std::fmt::Error>,
    {
        if self.is_none() {
            return Ok(());
        }

        let layers = &self.0;
        let render_layer =
            |element: ElementWriter<W, Initialized>, layer: &PaintLayer, cached: bool| {
                visit_layer(
                    element
                        .attr("fill", (&layer.paint,))?
                        .attr_if(
                            "fill-opacity",
                            layer.opacity,
                            layer.opacity != NormalizedF32::ONE,
                        )?
                        .attr_if(
                            "style",
                            (format_args!("mix-blend-mode:{}", layer.blend_mode),),
                            !layer.blend_mode.is_default(),
                        )?,
                    cached,
                )?
                .close()
            };

        if layers.len() == 1 {
            render_layer(
                ElementWriter::new(ctx.out, "path")?.write_attr("d", draw_single_layer)?,
                &layers[0],
                false,
            )?;

            return Ok(());
        }

        let path = Path::build(draw_cached_layer)?;
        let href = format_args!("#{}", path.iri());
        ctx.scene.resources.lock().get_or_add_resource(path.into());

        visit_group(ElementWriter::new(ctx.out, "g")?.attr_if(
            "style",
            "isolation:isolate",
            self.needs_isolation(),
        )?)?
        .content(|out| {
            layers.iter().try_for_each(|layer| {
                render_layer(
                    ElementWriter::new(out, "use")?.attr("href", (href,))?,
                    layer,
                    true,
                )
            })
        })?
        .close()?;

        Ok(())
    }
}

impl IntoResources for PaintStack {
    fn into_resources(self) -> Vec<Resource> {
        self.0
            .into_iter()
            .map(|x| x.paint.into_resources())
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        layout::Scene,
        test_utils::assert_xml,
    };
    use std::fmt::Write;

    #[test]
    fn renders_none() {
        assert_eq!(Paint::none().to_string(), "none");
    }

    #[test]
    fn is_none() {
        assert!(Paint::none().is_none());
        assert!(!Paint::color(Color::default()).is_none());
    }

    #[test]
    fn renders_color_paint() {
        assert_eq!(Paint::color(Color::rgb(1, 2, 3)).to_string(), "rgb(1,2,3)");
    }

    #[test]
    fn renders_linear_gradient_paint() {
        let lg = LinearGradient::new();
        assert_eq!(
            Paint::linear_gradient(lg.clone()).to_string(),
            format!(r#"url(#{})"#, lg.iri())
        );
    }

    #[test]
    fn renders_radial_gradient_paint() {
        let rg = RadialGradient::new();
        assert_eq!(
            Paint::radial_gradient(rg.clone()).to_string(),
            format!(r#"url(#{})"#, rg.iri())
        );
    }

    #[test]
    fn renders_image_paint() {
        let img = ImagePaint::new("test");
        let pattern = img.clone().into_pattern().unwrap();

        assert_eq!(
            Paint::image(img).to_string(),
            format!(r#"url(#{})"#, pattern.iri())
        );

        assert_xml(
            pattern.to_string(),
            format!(
                r#"
<pattern id="{}" width="1" height="1" patternContentUnits="objectBoundingBox">
    <image href="test" preserveAspectRatio="none" x="0" y="0" width="1" height="1" />
</pattern>
"#,
                pattern.iri()
            ),
        );
    }

    //

    #[test]
    fn paint_layer_defaults_to_none() {
        let layer = PaintLayer::default();
        assert!(layer.is_none());
        assert_eq!(layer.opacity, NormalizedF32::ONE);
        assert!(layer.blend_mode.is_default());
    }

    #[test]
    fn paint_layer_with_blend_and_opacity() {
        let layer = PaintLayer::from(Color::rgb(0, 0, 0))
            .blend_mode(BlendMode::Multiply)
            .opacity(0.5);

        assert_eq!(layer.opacity.get(), 0.5);
        assert_eq!(layer.blend_mode, BlendMode::Multiply);
    }

    //

    #[test]
    fn paint_stack_defaults_to_none() {
        assert!(PaintStack::default().is_none());
    }

    #[test]
    fn single_none_layer_is_none() {
        assert!(PaintStack::new([Paint::none()]).is_none());
    }

    #[test]
    fn renders_single_layer() {
        let stack = PaintStack::new([Color::rgb(0, 0, 0)]);
        let mut out = String::new();

        stack
            .render(
                &mut RenderContext::new(&mut out, &Scene::empty()),
                |out| out.write_str("path_data"),
                |out| out.write_str("path_data"),
                |layer, _| Ok(layer),
                |group| Ok(group),
            )
            .unwrap();

        assert_xml(out, r#"<path d="path_data" fill="rgb(0,0,0)" />"#);
    }

    #[test]
    fn visits_single_layer() {
        let stack = PaintStack::new([Color::rgb(0, 0, 0)]);
        let mut out = String::new();

        stack
            .render(
                &mut RenderContext::new(&mut out, &Scene::empty()),
                |out| out.write_str("path_data"),
                |out| out.write_str("path_data"),
                |layer, _| layer.attr("visited", "true"),
                |group| group.attr("visited", "true"),
            )
            .unwrap();

        assert_xml(
            out,
            r#"<path d="path_data" fill="rgb(0,0,0)" visited="true" />"#,
        );
    }

    #[test]
    fn renders_multiple_layers() {
        let stack = PaintStack::new([
            PaintLayer::from(Color::rgb(0, 0, 0)),
            PaintLayer::from(Color::rgb(1, 2, 3)).blend_mode(BlendMode::Multiply),
        ]);

        let mut out = String::new();
        let scene = Scene::empty();

        stack
            .render(
                &mut RenderContext::new(&mut out, &scene),
                |out| out.write_str("path_data"),
                |out| out.write_str("path_data"),
                |layer, _| Ok(layer),
                |group| Ok(group),
            )
            .unwrap();

        let path_iri = match scene.resources.lock().inner().get(0).unwrap() {
            Resource::Path(path) => path.iri(),
            _ => panic!("path not found"),
        };

        assert_xml(
            out,
            format!(
                r##"
<g style="isolation:isolate">
    <use href="#{path_iri}" fill="rgb(0,0,0)" />
    <use href="#{path_iri}" fill="rgb(1,2,3)" style="mix-blend-mode:multiply" />
</g>
"##,
            ),
        );
    }

    #[test]
    fn visits_group() {
        let stack = PaintStack::new([
            PaintLayer::from(Color::rgb(0, 0, 0)),
            PaintLayer::from(Color::rgb(1, 2, 3)).blend_mode(BlendMode::Multiply),
        ]);
        let mut out = String::new();
        let scene = Scene::empty();

        stack
            .render(
                &mut RenderContext::new(&mut out, &scene),
                |out| out.write_str("path_data"),
                |out| out.write_str("path_data"),
                |layer, _| Ok(layer),
                |group| group.attr("visited", "true"),
            )
            .unwrap();

        let path_iri = match scene.resources.lock().inner().get(0).unwrap() {
            Resource::Path(path) => path.iri(),
            _ => panic!("path not found"),
        };

        assert_xml(
            out,
            format!(
                r##"
<g style="isolation:isolate" visited="true">
    <use href="#{path_iri}" fill="rgb(0,0,0)" />
    <use href="#{path_iri}" fill="rgb(1,2,3)" style="mix-blend-mode:multiply" />
</g>
"##,
            ),
        );
    }

    #[test]
    fn visits_multiple_layers() {
        let stack = PaintStack::new([
            PaintLayer::from(Color::rgb(0, 0, 0)),
            PaintLayer::from(Color::rgb(1, 2, 3)).blend_mode(BlendMode::Multiply),
        ]);
        let mut out = String::new();
        let scene = Scene::empty();

        stack
            .render(
                &mut RenderContext::new(&mut out, &scene),
                |out| out.write_str("path_data"),
                |out| out.write_str("path_data"),
                |layer, _| layer.attr("visited", "true"),
                |group| Ok(group),
            )
            .unwrap();

        let path_iri = match scene.resources.lock().inner().get(0).unwrap() {
            Resource::Path(path) => path.iri(),
            _ => panic!("path not found"),
        };

        assert_xml(
            out,
            format!(
                r##"
<g style="isolation:isolate">
    <use href="#{path_iri}" fill="rgb(0,0,0)" visited="true" />
    <use href="#{path_iri}" fill="rgb(1,2,3)" style="mix-blend-mode:multiply" visited="true" />
</g>
"##,
            ),
        );
    }
}
