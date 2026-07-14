use crate::{
    layout::{
        ImageMeta,
        RenderContext,
        TextVectorizeError,
        Typography,
        text::TextMeta,
    },
    paint::{
        Appearance,
        Resource,
        ResourceIri,
        ScaledRadii,
        compute_scaled_radii,
        write_border_path,
        write_clip_path,
        write_fill_path,
    },
    primitives::{
        ClipPath,
        Rect,
    },
    utils::{
        ElementWriter,
        IsDefault,
    },
};
use enum_display::EnumDisplay;
use std::fmt::Write;
use thiserror::Error;

/// The error that may occur during vectorization of a scene.
#[derive(Debug, Error)]
pub enum VectorizeError {
    #[error("cannot vectorize an empty scene")]
    EmptyScene,
    #[error("scene does not have a valid size")]
    InvalidSize,
    #[error("failed to write to the output stream")]
    Write(#[from] std::fmt::Error),
    #[error("failed to vectorize text")]
    TextVectorize(#[from] TextVectorizeError),
}

#[derive(Debug, Clone, EnumDisplay)]
pub(crate) enum NodeKind {
    Block,
    Flex,
    Column,
    Row,
    #[cfg(feature = "grid")]
    Grid,
    #[display("Text: {0}")]
    Text(TextMeta),
    #[display("Image: {0:?}")]
    Image(ImageMeta),
}

impl NodeKind {
    pub(crate) fn is_atomic(&self) -> bool {
        matches!(self, NodeKind::Text(_) | NodeKind::Image(_))
    }
}

/// The single node in the scene graph.
#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) layout: taffy::Style,
    pub(crate) visual: Appearance,
    pub(crate) children: Vec<usize>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) typography: Typography,
    // computed
    pub(crate) cache: taffy::Cache,
    pub(crate) unrounded_layout: taffy::Layout,
    pub(crate) final_layout: taffy::Layout,
    pub(crate) scaled_radii: ScaledRadii,
}

impl Node {
    /// Creates a new [`Node`].
    ///
    /// # Arguments
    /// - `kind`: The [`NodeKind`] value.
    /// - `layout`: The layout style.
    /// - `visual`: The [`Appearance`] value.
    /// - `typography`: Optional [`Typography`] value.
    /// - `resources`: Resources referenced by the node.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn new(
        kind: NodeKind,
        layout: taffy::Style,
        visual: Appearance,
        typography: Option<Typography>,
        resources: Vec<Resource>,
    ) -> Self {
        Self {
            kind,
            layout,
            visual,
            children: Vec::new(),
            resources,
            typography: typography.unwrap_or(Typography::default()),
            cache: taffy::Cache::new(),
            unrounded_layout: taffy::Layout::with_order(0),
            final_layout: taffy::Layout::with_order(0),
            scaled_radii: ScaledRadii::default(),
        }
    }

    /// Applies layout-dependent visual effects after layout resolution.
    pub(crate) fn apply_layout_effects(&mut self) {
        self.scaled_radii = compute_scaled_radii(
            self.visual.corner_radius,
            self.final_layout.size.width,
            self.final_layout.size.height,
        );
    }

    /// Returns `true` if the node has a visible border.
    fn has_border(&self) -> bool {
        let taffy::Rect {
            top,
            right,
            bottom,
            left,
        } = self.final_layout.border;
        !self.visual.border.is_none() && (top + right + bottom + left) > 0.0
    }

    /// Returns `true` if any corner radius is non-zero.
    fn has_radius(&self) -> bool {
        let radius = self.visual.corner_radius;
        !radius.top_left.is_zero()
            || !radius.top_right.is_zero()
            || !radius.bottom_left.is_zero()
            || !radius.bottom_right.is_zero()
    }

    /// Determines whether clipping should be applied on each axis.
    fn should_clip(&self) -> (bool, bool) {
        let clip_x = self.layout.overflow.x == taffy::Overflow::Hidden;
        let clip_y = self.layout.overflow.y == taffy::Overflow::Hidden;
        (clip_x, clip_y)
    }

    /// Opens the root SVG group for a block-level node.
    fn open_block_group<T>(&self, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        let taffy::Layout {
            location: taffy::Point { x, y },
            size: taffy::Size { width, height },
            ..
        } = self.final_layout;

        ElementWriter::new(ctx.out, "g")?
            .attr_if("opacity", self.visual.opacity, self.visual.opacity != 1.0)?
            .attr_if(
                "filter",
                (format_args!("url(#{})", self.visual.filter.iri()),),
                !self.visual.filter.is_default(),
            )?
            .attr_if(
                "style",
                (format_args!("mix-blend-mode:{}", self.visual.blend_mode),),
                !self.visual.blend_mode.is_default(),
            )?
            .write(|out| {
                self.visual
                    .transform
                    .write(out, (0.0, 0.0), (x, y), (width, height))
            })?
            .open()
            .map(|_| ())
            .map_err(Into::into)
    }

    /// Renders the background fill of a block-level node.
    fn render_block_background<T>(&self, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        let taffy::Size {
            width: w,
            height: h,
        } = self.final_layout.size;
        let radius = self.scaled_radii;

        self.visual
            .background
            .render(
                ctx,
                |out| write_fill_path(out, w, h, radius),
                |out| write_fill_path(out, w, h, radius),
                |x, _| Ok(x),
                |x| Ok(x),
            )
            .map_err(Into::into)
    }

    /// Renders the border of a block-level node if present.
    fn render_block_border<T>(&self, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        if !self.has_border() {
            return Ok(());
        }

        let taffy::Size {
            width: w,
            height: h,
        } = self.final_layout.size;
        let border = Rect::from(self.final_layout.border);

        self.visual
            .border
            .render(
                ctx,
                |out| write_border_path(out, w, h, self.scaled_radii, border),
                |out| write_border_path(out, w, h, self.scaled_radii, border),
                |layer, _| layer.attrs([("fill-rule", "evenodd"), ("clip-rule", "evenodd")]),
                |x| Ok(x),
            )
            .map_err(Into::into)
    }

    /// Opens a clipping group for the node when overflow clipping is required.
    fn open_block_clip<T>(
        &self,
        ctx: &mut RenderContext<T>,
        (clip_x, clip_y): (bool, bool),
    ) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        if !clip_x && !clip_y {
            return Ok(());
        }

        let taffy::Size { width, height } = self.final_layout.size;
        let radius = self.scaled_radii;
        let border = Rect::from(self.final_layout.border);
        let clip = ClipPath::build(|out| {
            ElementWriter::new(out, "path")?
                .write_attr("d", |out| {
                    write_clip_path(
                        out,
                        width,
                        height,
                        radius,
                        border,
                        clip_x,
                        clip_y,
                        ctx.scene_size,
                    )
                })?
                .close()
        })?;

        ElementWriter::new(ctx.out, "g")?
            .attr("clip-path", (format_args!("url(#{})", clip.iri()),))?
            .open()?;

        ctx.scene.resources.lock().get_or_add_resource(clip.into());

        Ok(())
    }

    /// Closes the SVG groups opened for a block-level node.
    fn close_block_group<T>(clipped: bool, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        if clipped {
            ElementWriter::close_tag(ctx.out, "g")?;
        }

        ElementWriter::close_tag(ctx.out, "g").map_err(Into::into)
    }

    /// Starts SVG emission for the node.
    pub(crate) fn pre_emit<T>(&self, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        match &self.kind {
            NodeKind::Block | NodeKind::Flex | NodeKind::Column | NodeKind::Row => {
                self.open_block_group(ctx)?;
                self.render_block_background(ctx)?;
                self.render_block_border(ctx)?;
                self.open_block_clip(ctx, self.should_clip())?;
            }
            //
            #[cfg(feature = "grid")]
            NodeKind::Grid => {
                self.open_block_group(ctx)?;
                self.render_block_background(ctx)?;
                self.render_block_border(ctx)?;
                self.open_block_clip(ctx, self.should_clip())?;
            }
            //
            NodeKind::Text(text) => {
                self.open_block_group(ctx)?;
                self.render_block_background(ctx)?;
                text.render(ctx, self.final_layout)?;
                Self::close_block_group(false, ctx)?;
            }
            //
            NodeKind::Image(image) => {
                let has_radius = self.has_radius();

                self.open_block_group(ctx)?;
                self.render_block_background(ctx)?;
                self.render_block_border(ctx)?;
                self.open_block_clip(ctx, (has_radius, has_radius))?;
                image.render(ctx, self.final_layout)?;
                Self::close_block_group(has_radius, ctx)?;
            }
        };

        Ok(())
    }

    /// Ends SVG emission for the node.
    pub(crate) fn post_emit<T>(&self, ctx: &mut RenderContext<T>) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        match &self.kind {
            NodeKind::Block | NodeKind::Flex | NodeKind::Column | NodeKind::Row => {
                Self::close_block_group(
                    self.layout.overflow.x == taffy::Overflow::Hidden
                        || self.layout.overflow.y == taffy::Overflow::Hidden,
                    ctx,
                )?;
            }
            //
            #[cfg(feature = "grid")]
            NodeKind::Grid => {
                Self::close_block_group(
                    self.layout.overflow.x == taffy::Overflow::Hidden
                        || self.layout.overflow.y == taffy::Overflow::Hidden,
                    ctx,
                )?;
            }
            //
            NodeKind::Text(_) | NodeKind::Image(_) => {}
        }

        Ok(())
    }
}
