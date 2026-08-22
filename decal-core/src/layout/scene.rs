#[cfg(all(not(target_arch = "wasm32"), feature = "fetch"))]
use crate::layout::fetch::fetch_image_sync;
use crate::{
    layout::{
        CachedImage,
        FontRegistry,
        ImageCache,
        ImageOptions,
        Node,
        NodeId,
        NodeKind,
        RasterizeOptions,
        RenderContext,
        SvgDimensions,
        Typography,
        VectorizeError,
        VectorizeOptions,
    },
    paint::Resources,
    prelude::ViewBox,
    primitives::Size,
    utils::ElementWriter,
};
use hashbrown::HashSet;
use parking_lot::Mutex;
use resvg::render;
use smallvec::SmallVec;
use std::{
    fmt::Write,
    sync::Arc,
};
#[cfg(feature = "grid")]
use taffy::compute_grid_layout;
use taffy::{
    CacheTree,
    LayoutPartialTree,
    PrintTree,
    RoundTree,
    TraversePartialTree,
    TraverseTree,
    compute_block_layout,
    compute_cached_layout,
    compute_flexbox_layout,
    compute_leaf_layout,
    compute_root_layout,
    prelude::TaffyMaxContent,
    print_tree,
    round_layout,
};
use thiserror::Error;
use tiny_skia::Pixmap;
use usvg::{
    ImageHrefResolver,
    ImageKind,
    Tree,
};

const ROOT_ID: usize = 0;
const INLINE_FRAG_CASCADE: usize = 16;

/// The error that may occur during rasterization of a scene.
#[derive(Debug, Error)]
pub enum RasterizeError {
    #[error("failed to vectorize")]
    Vectorize(#[from] VectorizeError),
    #[error("failed to write to the output stream")]
    Write(#[from] std::fmt::Error),
    #[error("failed to parse svg")]
    Parse(#[from] usvg::Error),
    #[error("failed to allocate pixmap")]
    PixmapAlloc,
}

/// The central scene container responsible for layout, rendering, and resource
/// management.
#[derive(Debug)]
pub struct Scene {
    pub(crate) fonts: Arc<Mutex<FontRegistry>>,
    pub(crate) resources: Mutex<Resources>,
    pub(crate) image_refs: HashSet<String>,
    nodes: Vec<Node>,
}

impl Scene {
    /// Creates a new scene with the given root node.
    ///
    /// # Arguments
    /// - `root`: The root [`Node`] of the scene.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new(root: Node) -> Self {
        let mut resources = Resources::default();
        let mut image_refs = HashSet::new();

        if let NodeKind::Image(ref meta) = root.kind
            && let Some(href) = meta.source.as_href()
        {
            image_refs.insert(href.to_string());
        }

        for resource in &root.resources {
            resources.get_or_add_resource(resource.clone());
            image_refs.extend(resource.image_refs());
        }

        Self {
            fonts: Arc::new(Mutex::new(FontRegistry::new())),
            resources: Mutex::new(resources),
            nodes: vec![root],
            image_refs,
        }
    }

    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            fonts: Arc::new(Mutex::new(FontRegistry::new())),
            resources: Mutex::new(Resources::default()),
            nodes: Vec::new(),
            image_refs: HashSet::new(),
        }
    }

    /// Returns the [`NodeId`] of the root node.
    ///
    /// # Returns
    /// - [`NodeId`] corresponding to the root.
    pub fn root_id(&self) -> NodeId {
        ROOT_ID.into()
    }

    /// Returns the total number of nodes in the scene.
    ///
    /// # Returns
    /// - The number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Appends a child node to the scene under the given parent node.
    ///
    /// # Arguments
    /// - `parent_id`: The [`NodeId`] of the parent node.
    /// - `child`: The child [`Node`] to append.
    ///
    /// # Returns
    /// - [`NodeId`] of the newly added child.
    ///
    /// # Warning
    /// Panics if the parent node is atomic and cannot contain children.
    pub fn append_child(&mut self, parent_id: NodeId, mut child: Node) -> NodeId {
        self.assert_non_atomic(parent_id);
        let parent = &self.nodes[parent_id];
        child.typography.cascade_from(&parent.typography);

        // copy the resolved typography into its metadata
        if let NodeKind::Text(ref mut meta) = child.kind {
            meta.typography(child.typography.clone());
        }

        // collect image href
        if let NodeKind::Image(ref meta) = child.kind
            && let Some(href) = meta.source.as_href()
        {
            self.image_refs.insert(href.to_string());
        }

        {
            let mut resources = self.resources.lock();
            for resource in &child.resources {
                resources.get_or_add_resource(resource.clone());
                self.image_refs.extend(resource.image_refs());
            }
        }

        let child_id = self.nodes.len();
        self.nodes.push(child);
        self.nodes[parent_id].children.push(child_id);

        child_id.into()
    }

    /// Appends an entire scene as a subtree to the current scene under the
    /// given parent node.
    ///
    /// All node indices and typography cascades are adjusted accordingly.
    ///
    /// # Arguments
    /// - `parent_id`: The [`NodeId`] of the parent node.
    /// - `scene`: The scene to append.
    pub fn append_scene(&mut self, parent_id: NodeId, mut scene: Scene) {
        if scene.nodes.is_empty() {
            return;
        }

        self.assert_non_atomic(parent_id);

        let parent_typography = &self.nodes[parent_id].typography;
        cascade_typography_subtree(&mut scene.nodes, parent_typography);

        let root_id = self.nodes.len(); // scene root node
        self.nodes.reserve(scene.nodes.len()); // pre-allocation
        self.nodes[parent_id].children.push(root_id);

        for mut node in scene.nodes {
            // update child indices after adding them to main arena
            for child_id in node.children.iter_mut() {
                *child_id += root_id;
            }

            // register resources
            {
                let mut resources = self.resources.lock();
                for resource in &node.resources {
                    resources.get_or_add_resource(resource.clone());
                }
            }

            self.nodes.push(node);
        }

        self.image_refs.extend(scene.image_refs);
    }

    /// Prints the scene graph in a tree format.
    #[allow(dead_code)]
    pub(crate) fn print_tree(&self) {
        print_tree(self, taffy::NodeId::from(ROOT_ID));
    }

    /// Returns the list of external image URLs present in this scene.
    pub(crate) fn image_refs(&self) -> &HashSet<String> {
        &self.image_refs
    }

    /// Streams the vectorized SVG representation into the given output writer.
    ///
    /// # Arguments
    /// - `out`: The output writer.
    /// - `options`: The [`VectorizeOptions`] value.
    ///
    /// # Returns
    /// - Scene size on success.
    /// - [`VectorizeError`] on failure.
    pub(crate) fn stream_vector<T>(
        &self,
        out: &mut T,
        options: &VectorizeOptions,
    ) -> Result<Size<f32>, VectorizeError>
    where
        T: Write,
    {
        if self.nodes.is_empty() {
            return Err(VectorizeError::EmptyScene);
        }

        let root = &self.nodes[ROOT_ID];
        let size = Size::from(root.final_layout.size);

        if size.width == 0.0 || size.height == 0.0 {
            return Err(VectorizeError::InvalidSize);
        }

        let view_box = ViewBox::new(0.0, 0.0, size.width, size.height);
        let mut svg = ElementWriter::new(out, "svg")?
            .attr_if(
                "xmlns",
                "http://www.w3.org/2000/svg",
                !options.omit_svg_xmlns,
            )?
            .attr("viewBox", (view_box,))?;

        match &options.svg_dimensions {
            SvgDimensions::Omit => {}
            SvgDimensions::Layout => {
                svg = svg.attrs([("width", size.width), ("height", size.height)])?;
            }
            SvgDimensions::Custom { width, height } => {
                svg = svg.attrs([("width", width.as_str()), ("height", height.as_str())])?;
            }
        };

        svg.open()?;

        //

        self.emit_node(
            &mut RenderContext {
                out,
                scene: self,
                scene_size: size,
            },
            None,
            None,
        )?;

        //

        let resources = self.resources.lock();

        if !resources.is_empty() {
            ElementWriter::new(out, "defs")?
                .content(|out| out.write_fmt(format_args!("{resources}")))?
                .close()?;
        }

        ElementWriter::close_tag(out, "svg")?;

        Ok(size)
    }

    /// Vectorizes the scene into an SVG string.
    ///
    /// # Arguments
    /// - `options`: The [`VectorizeOptions`] value.
    ///
    /// # Returns
    /// - On success, a tuple containing:
    ///     - [`String`]: The SVG string.
    ///     - [`Size<f32>`]: Scene size.
    /// - [`VectorizeError`] on failure.
    pub(crate) fn vectorize(
        &self,
        options: &VectorizeOptions,
    ) -> Result<(String, Size<f32>), VectorizeError> {
        let mut out = String::new();
        self.stream_vector(&mut out, options)
            .map(|scene_size| (out, scene_size))
    }

    /// Rasterizes the scene into a [`Pixmap`].
    ///
    /// # Arguments
    /// - `image_cache`: Shared image cache.
    /// - `options`: The [`RasterizeOptions`] value.
    ///
    /// # Returns
    /// - On success, a tuple containing:
    ///     - [`Pixmap`]: Pixmap containing the image data.
    ///     - [`Size<f32>`]: Scene size.
    /// - [`RasterizeError`] on failure.
    pub(crate) fn rasterize(
        &self,
        image_cache: &ImageCache,
        options: &RasterizeOptions,
    ) -> Result<(Pixmap, Size<f32>), RasterizeError> {
        let tf = options.root_transform;
        let mut usvg_options = usvg::Options {
            shape_rendering: options.shape_rendering,
            text_rendering: options.text_rendering,
            image_rendering: options.image_rendering,
            image_href_resolver: ImageHrefResolver {
                resolve_string: Box::new(move |href: &str, usvg_opts: &usvg::Options| {
                    fetch_image_cached(image_cache, href, &options.image, usvg_opts)
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        if let Some(ref resolve_data) = options.image.href_data_resolver {
            usvg_options.image_href_resolver.resolve_data = Box::new(
                move |mime: &str, data: Arc<Vec<u8>>, opts: &usvg::Options| {
                    resolve_data(mime, data, opts)
                },
            );
        }

        let (svg, size) = self.vectorize(&options.vectorize_options)?;
        let tree = Tree::from_str(&svg, &usvg_options).map_err(RasterizeError::Parse)?;
        let mut pixmap = Pixmap::new(size.width() as u32, size.height() as u32)
            .ok_or(RasterizeError::PixmapAlloc)?;

        render(&tree, tf, &mut pixmap.as_mut());

        if options.debug {
            let mut bboxes = Vec::new();
            let mut stroke_bboxes = Vec::new();

            collect_bboxes(tree.root(), &mut bboxes, &mut stroke_bboxes);

            let stroke = tiny_skia::Stroke::default();
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(224, 16, 0, 195);

            for bbox in bboxes {
                let path = tiny_skia::PathBuilder::from_rect(bbox);
                pixmap.stroke_path(&path, &paint, &stroke, tf, None);
            }

            paint.set_color_rgba8(0, 45, 255, 127);

            for bbox in stroke_bboxes {
                let path = tiny_skia::PathBuilder::from_rect(bbox);
                pixmap.stroke_path(&path, &paint, &stroke, tf, None);
            }
        }

        Ok((pixmap, size))
    }

    /// Computes layout for all nodes in the scene.
    pub(crate) fn compute_layout(&mut self) {
        let root_id = taffy::NodeId::from(self.root_id());
        compute_root_layout(self, root_id, taffy::Size::MAX_CONTENT);
        round_layout(self, root_id);
    }

    /// Sets the font registry used for layout and rendering.
    ///
    /// # Arguments
    /// - `fonts`: The next font registry.
    pub(crate) fn set_fonts(&mut self, fonts: Arc<Mutex<FontRegistry>>) {
        self.fonts = fonts;
    }

    /// Panics if the node with the given `id` is atomic (cannot have children).
    ///
    /// Note: This is a safety check. The macro should prevent adding children
    /// to atomic nodes at compile time.
    fn assert_non_atomic(&self, id: NodeId) {
        if self.nodes[id].kind.is_atomic() {
            panic!("node with id {id} is atomic and cannot contain children");
        }
    }

    #[inline(always)]
    fn node_from_id(&self, node_id: taffy::NodeId) -> &Node {
        &self.nodes[usize::from(node_id)]
    }

    #[inline(always)]
    fn node_from_id_mut(&mut self, node_id: taffy::NodeId) -> &mut Node {
        &mut self.nodes[usize::from(node_id)]
    }

    /// Emits a node and its subtree into the render context.
    pub(crate) fn emit_node<T>(
        &self,
        ctx: &mut RenderContext<T>,
        node_id: Option<taffy::NodeId>,
        stop_at: Option<taffy::NodeId>,
    ) -> Result<(), VectorizeError>
    where
        T: Write,
    {
        if stop_at.is_some() && node_id == stop_at {
            return Ok(());
        }

        let node_idx = node_id.map(usize::from).unwrap_or(ROOT_ID);
        let node = &self.nodes[node_idx];

        if node.visual.visible && !matches!(node.layout.display, taffy::Display::None) {
            node.pre_emit(ctx)?;

            for child_id in &node.children {
                self.emit_node(ctx, Some(taffy::NodeId::from(*child_id)), stop_at)?;
            }

            node.post_emit(ctx)?;
        }

        Ok(())
    }
}

/// Resolves and caches external images referenced by `href`.
///
/// # Arguments
/// - `image_cache`: The shared image cache.
/// - `href`: The image reference.
/// - `opts`: The [`ImageOptions`] value.
/// - `usvg_opts`: The [`Options`] value.
///
/// # Returns
/// - `Some(ImageKind)` on success
/// - `None` during failure
///
/// [`Options`]: usvg::Options
fn fetch_image_cached(
    image_cache: &ImageCache,
    href: &str,
    opts: &ImageOptions,
    usvg_opts: &usvg::Options,
) -> Option<ImageKind> {
    if let Some(image) = image_cache.lock().get(href) {
        return image.clone().into_image_kind(usvg_opts);
    }

    let image = if let Some(resolve) = &opts.href_string_resolver {
        resolve(href, usvg_opts).map(|img| CachedImage::Resolved(Box::new(img)))
    } else {
        // blocking fetch
        #[cfg(all(not(target_arch = "wasm32"), feature = "fetch"))]
        {
            fetch_image_sync(href)
        }

        // this will only be called when the `fetch` feature is disabled and the user
        // hasn't provided a href resolver; image will already be resolved and preloaded
        // into the cache when using the `fetch-async` feature or targeting WASM
        #[cfg(any(target_arch = "wasm32", not(feature = "fetch")))]
        {
            log::warn!(
                "failed to get image `{href}`: `fetch` feature is disabled and image resolver is not provided",
            );

            None
        }
    };

    if let Some(kind) = image.clone() {
        image_cache.lock().push(href.to_string(), kind);
    }

    image.and_then(|img| img.into_image_kind(usvg_opts))
}

/// Cascades typography values through a subtree rooted at the first node.
///
/// # Arguments
/// - `nodes`: The node list.
/// - `parent_typography`: The parent node's [`Typography`].
fn cascade_typography_subtree(nodes: &mut [Node], parent_typography: &Typography) {
    let mut stack: SmallVec<[(usize, Typography); INLINE_FRAG_CASCADE]> = SmallVec::new();
    stack.push((ROOT_ID, parent_typography.clone()));

    while let Some((idx, mut parent)) = stack.pop() {
        let node = &mut nodes[idx];
        node.typography.cascade_from(&parent);

        if let NodeKind::Text(ref mut meta) = node.kind {
            meta.typography(node.typography.clone());
        }

        if !node.children.is_empty() {
            parent = node.typography.clone();
        }

        for &child in &node.children {
            stack.push((child, parent.clone()));
        }
    }
}

/// Collects bounding boxes and stroke bounding boxes for debugging output.
fn collect_bboxes(
    parent: &usvg::Group,
    bboxes: &mut Vec<usvg::Rect>,
    stroke_bboxes: &mut Vec<usvg::Rect>,
) {
    for node in parent.children() {
        if let usvg::Node::Group(group) = node {
            collect_bboxes(group, bboxes, stroke_bboxes);
        }

        let bbox = node.abs_bounding_box();
        bboxes.push(bbox);

        let stroke_bbox = node.abs_stroke_bounding_box();
        if bbox != stroke_bbox {
            stroke_bboxes.push(stroke_bbox);
        }
    }
}

impl std::ops::Index<NodeId> for Vec<Node> {
    type Output = Node;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self[usize::from(id)]
    }
}

impl std::ops::IndexMut<NodeId> for Vec<Node> {
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        &mut self[usize::from(id)]
    }
}

mod sealed {
    pub struct ChildIter<'a>(pub std::slice::Iter<'a, usize>);
}

impl Iterator for sealed::ChildIter<'_> {
    type Item = taffy::NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(taffy::NodeId::from)
    }
}

impl TraversePartialTree for Scene {
    type ChildIter<'a> = sealed::ChildIter<'a>;

    fn child_ids(&self, node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        sealed::ChildIter(self.node_from_id(node_id).children.iter())
    }

    fn child_count(&self, node_id: taffy::NodeId) -> usize {
        self.node_from_id(node_id).children.len()
    }

    fn get_child_id(&self, node_id: taffy::NodeId, index: usize) -> taffy::NodeId {
        taffy::NodeId::from(self.node_from_id(node_id).children[index])
    }
}

impl TraverseTree for Scene {}

impl LayoutPartialTree for Scene {
    type CoreContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type CustomIdent = String;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        &self.node_from_id(node_id).layout
    }

    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.node_from_id_mut(node_id).unrounded_layout = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::tree::LayoutInput,
    ) -> taffy::tree::LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let node = &mut tree.nodes[usize::from(node_id)];
            match node.kind {
                NodeKind::Block => compute_block_layout(tree, node_id, inputs),
                NodeKind::Flex | NodeKind::Column | NodeKind::Row => {
                    compute_flexbox_layout(tree, node_id, inputs)
                }
                #[cfg(feature = "grid")]
                NodeKind::Grid => compute_grid_layout(tree, node_id, inputs),
                NodeKind::Text(ref mut meta) => compute_leaf_layout(
                    inputs,
                    &node.layout,
                    |_val, _basis| 0.0,
                    |known_dimensions, available_space| {
                        meta.measure(known_dimensions, available_space, tree.fonts.clone())
                    },
                ),
                NodeKind::Image(ref mut meta) => compute_leaf_layout(
                    inputs,
                    &node.layout,
                    |_val, _basis| 0.0,
                    |known_dimensions, _available_space| meta.measure(known_dimensions),
                ),
            }
        })
    }
}

impl CacheTree for Scene {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.node_from_id(node_id)
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.node_from_id_mut(node_id).cache.store(
            known_dimensions,
            available_space,
            run_mode,
            layout_output,
        )
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.node_from_id_mut(node_id).cache.clear();
    }
}

impl taffy::LayoutBlockContainer for Scene {
    type BlockContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        &self.node_from_id(node_id).layout
    }

    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        &self.node_from_id(child_node_id).layout
    }
}

impl taffy::LayoutFlexboxContainer for Scene {
    type FlexboxContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        &self.node_from_id(node_id).layout
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.node_from_id(child_node_id).layout
    }
}

#[cfg(feature = "grid")]
impl taffy::LayoutGridContainer for Scene {
    type GridContainerStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a taffy::Style
    where
        Self: 'a;

    fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
        &self.node_from_id(node_id).layout
    }

    fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
        &self.node_from_id(child_node_id).layout
    }
}

impl RoundTree for Scene {
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.node_from_id(node_id).unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        let node = self.node_from_id_mut(node_id);
        node.final_layout = *layout;
        node.apply_layout_effects();
    }
}

impl PrintTree for Scene {
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        match self.node_from_id(node_id).kind {
            NodeKind::Block => "BLOCK",
            NodeKind::Flex => "FLEX",
            NodeKind::Column => "COLUMN",
            NodeKind::Row => "ROW",
            #[cfg(feature = "grid")]
            NodeKind::Grid => "GRID",
            NodeKind::Text(_) => "TEXT",
            NodeKind::Image(_) => "IMAGE",
        }
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.node_from_id(node_id).final_layout
    }
}
