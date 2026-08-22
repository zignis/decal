/// Controls how child nodes are aligned within a container.
///
/// - For flexbox, this aligns items along the cross axis.
/// - For grid, this aligns items along the block axis.
#[derive(Debug, Copy, Clone)]
pub enum AlignItems {
    /// Items are packed toward the start of the axis.
    Start,
    /// Items are packed toward the end of the axis.
    End,
    /// Items are packed toward the flex-relative start of the axis.
    ///
    /// For flex containers with reversed directions (`row-reverse` or
    /// `column-reverse`), this is equivalent to [`End`]. Otherwise, it behaves
    /// the same as [`Start`].
    ///
    /// [`End`]: Self::End
    /// [`Start`]: Self::Start
    FlexStart,
    /// Items are packed toward the flex-relative end of the axis.
    ///
    /// For flex containers with reversed directions, this is equivalent to
    /// [`Start`]. Otherwise, it behaves the same as [`End`].
    ///
    /// [`End`]: Self::End
    /// [`Start`]: Self::Start
    FlexEnd,
    /// Items are centered along the axis.
    Center,
    /// Items are aligned such that their text baselines line up.
    Baseline,
    /// Items stretch to fill the available space.
    Stretch,
}

/// Controls how space is distributed between and around child nodes.
///
/// - For flexbox, this aligns content along the cross axis.
/// - For grid, this aligns content along the block axis.
#[derive(Debug, Copy, Clone)]
pub enum AlignContent {
    /// Items are packed toward the start of the axis.
    Start,
    /// Items are packed toward the end of the axis.
    End,
    /// Items are packed toward the flex-relative start of the axis.
    ///
    /// For reversed flex directions, this behaves like [`End`].
    ///
    /// [`End`]: Self::End
    FlexStart,
    /// Items are packed toward the flex-relative end of the axis.
    ///
    /// For reversed flex directions, this behaves like [`Start`].
    ///
    /// [`Start`]: Self::Start
    FlexEnd,
    /// Items are centered along the axis.
    Center,
    /// Items stretch to fill the available space.
    Stretch,
    /// Items are evenly distributed with no space at the edges.
    SpaceBetween,
    /// Items are evenly distributed with equal space around each item.
    SpaceEvenly,
    /// Items are evenly distributed with half-sized space at the edges.
    SpaceAround,
}

/// Controls how space is distributed along the main axis in flexbox or the
/// inline axis in grid.
///
/// Alias of [`AlignContent`].
pub type JustifyContent = AlignContent;

/// Controls how child nodes are aligned along the inline axis in grid.
///
/// This does not apply to flexbox and is ignored on flex containers.
///
/// Alias of [`AlignItems`].
pub type JustifyItems = AlignItems;

/// Controls alignment of an individual node along the cross or block axis.
///
/// This overrides the parent container's [`AlignItems`] value.
///
/// Alias of [`AlignItems`].
pub type AlignSelf = AlignItems;

/// Controls alignment of an individual node along the inline axis.
///
/// This overrides the parent container's [`JustifyItems`] value and does not
/// apply to flexbox.
///
/// Alias of [`AlignItems`].
pub type JustifySelf = AlignItems;

impl From<AlignItems> for taffy::AlignItems {
    #[inline]
    fn from(alignment: AlignItems) -> Self {
        match alignment {
            AlignItems::Start => taffy::AlignItems::Start,
            AlignItems::End => taffy::AlignItems::End,
            AlignItems::FlexStart => taffy::AlignItems::FlexStart,
            AlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::Baseline => taffy::AlignItems::Baseline,
            AlignItems::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

impl From<AlignContent> for taffy::AlignContent {
    #[inline]
    fn from(alignment: AlignContent) -> Self {
        match alignment {
            AlignContent::Start => taffy::AlignContent::Start,
            AlignContent::End => taffy::AlignContent::End,
            AlignContent::FlexStart => taffy::AlignContent::FlexStart,
            AlignContent::FlexEnd => taffy::AlignContent::FlexEnd,
            AlignContent::Center => taffy::AlignContent::Center,
            AlignContent::Stretch => taffy::AlignContent::Stretch,
            AlignContent::SpaceBetween => taffy::AlignContent::SpaceBetween,
            AlignContent::SpaceAround => taffy::AlignContent::SpaceAround,
            AlignContent::SpaceEvenly => taffy::AlignContent::SpaceEvenly,
        }
    }
}
