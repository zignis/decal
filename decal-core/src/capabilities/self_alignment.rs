use super::Drawable;
#[cfg(feature = "grid")]
use crate::primitives::JustifySelf;
use crate::primitives::{
    AlignSelf,
    IntoOptionalLength,
};
#[cfg(feature = "grid")]
use taffy::Line;
#[cfg(feature = "grid")]
use taffy::style::GridPlacement;

/// Capability for configuring self-alignment and flex item properties on a
/// node.
pub trait SelfAlignment: Drawable {
    /// Sets how the node is aligned along the cross axis within its container.
    ///
    /// # Arguments
    /// - `value`: The [`AlignSelf`] behavior.
    ///
    /// # Returns
    /// - [`Self`]
    fn align_self<T>(mut self, value: T) -> Self
    where
        T: Into<Option<AlignSelf>>,
    {
        self.layout_mut().align_self = value.into().map(Into::into);
        self
    }

    /// Sets the initial main size of the node before flexing is applied.
    ///
    /// # Arguments
    /// - `value`: The flex basis length convertible using
    ///   [`IntoOptionalLength`].
    ///
    /// # Returns
    /// - [`Self`]
    fn flex_basis<T>(mut self, value: T) -> Self
    where
        T: IntoOptionalLength,
    {
        self.layout_mut().flex_basis = value
            .into_optional_length()
            .map(Into::into)
            .unwrap_or(taffy::Dimension::auto());
        self
    }

    /// Sets the flex grow factor of the node.
    ///
    /// # Arguments
    /// - `value`: The growth factor controlling how much the node expands
    ///   relative to siblings.
    ///
    /// # Returns
    /// - [`Self`]
    fn flex_grow<T>(mut self, value: T) -> Self
    where
        T: Into<Option<f32>>,
    {
        self.layout_mut().flex_grow = value.into().unwrap_or(0.0);
        self
    }

    /// Sets the flex shrink factor of the node.
    ///
    /// # Arguments
    /// - `value`: The shrink factor controlling how the node contracts when
    ///   space is limited.
    ///
    /// # Returns
    /// - [`Self`]
    fn flex_shrink<T>(mut self, value: T) -> Self
    where
        T: Into<Option<f32>>,
    {
        self.layout_mut().flex_shrink = value.into().unwrap_or(1.0);
        self
    }

    /// Sets how the node is aligned along the main axis within its container.
    ///
    /// # Arguments
    /// - `value`: The [`JustifySelf`] behavior.
    ///
    /// # Returns
    /// - [`Self`]
    #[cfg(feature = "grid")]
    fn justify_self<T>(mut self, value: T) -> Self
    where
        T: Into<Option<JustifySelf>>,
    {
        self.layout_mut().justify_self = value.into().map(Into::into);
        self
    }

    /// Sets the grid row lines where the item should start and end.
    ///
    /// # Arguments
    /// - `value`: The starting and ending row lines for the grid item.
    ///
    /// # Returns
    /// - [`Self`]
    #[cfg(feature = "grid")]
    fn grid_row(mut self, value: Line<GridPlacement<String>>) -> Self {
        self.layout_mut().grid_row = value;
        self
    }

    /// Sets the grid column lines where the item should start and end.
    ///
    /// # Arguments
    /// - `value`: The starting and ending column lines for the grid item.
    ///
    /// # Returns
    /// - [`Self`]
    #[cfg(feature = "grid")]
    fn grid_column(mut self, value: Line<GridPlacement<String>>) -> Self {
        self.layout_mut().grid_column = value;
        self
    }
}
