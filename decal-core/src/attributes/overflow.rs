use crate::primitives::{
    Overflow,
    Point,
};

/// Type alias representing overflow behavior along the horizontal and vertical
/// axes.
type OverflowXY = Point<Overflow>;

/// Converts an internal overflow point into the corresponding Taffy overflow
/// representation.
impl From<OverflowXY> for taffy::Point<taffy::Overflow> {
    /// Maps each axis overflow value into its Taffy equivalent.
    #[inline]
    fn from(point: OverflowXY) -> Self {
        taffy::Point {
            x: point.x.into(),
            y: point.y.into(),
        }
    }
}

/// Conversion trait for values that can be interpreted as two-axis overflow
/// behavior.
pub trait IntoOverflow {
    /// Converts the value into a horizontal and vertical overflow definition.
    ///
    /// # Returns
    /// - `Some(OverflowXY)` when the value can be expanded into axis-specific
    ///   overflow values.
    /// - `None` when the value semantically represents the absence of overflow
    ///   configuration.
    fn into_overflow(self) -> Option<OverflowXY>;
}

/// Identity conversion for an optional overflow definition.
impl IntoOverflow for Option<OverflowXY> {
    /// Returns the contained overflow definition without modification.
    #[inline]
    fn into_overflow(self) -> Option<OverflowXY> {
        self
    }
}

/// Wraps an existing overflow definition into an optional value.
impl IntoOverflow for OverflowXY {
    /// Converts a concrete overflow definition into `Some(OverflowXY)`.
    #[inline]
    fn into_overflow(self) -> Option<OverflowXY> {
        Some(self)
    }
}

/// Expands a single overflow value into both horizontal and vertical axes.
impl IntoOverflow for Overflow {
    /// Uses the same overflow behavior for both axes.
    #[inline]
    fn into_overflow(self) -> Option<OverflowXY> {
        Some(Point { x: self, y: self })
    }
}

/// Converts a two-value tuple into explicit horizontal and vertical overflow
/// behavior.
impl IntoOverflow for (Overflow, Overflow) {
    /// Interprets the tuple as `(x, y)` overflow values.
    #[inline]
    fn into_overflow(self) -> Option<OverflowXY> {
        Some(Point {
            x: self.0,
            y: self.1,
        })
    }
}

/// Array-based shorthand for two-axis overflow definitions.
impl IntoOverflow for [Overflow; 2] {
    /// Delegates to the two-element tuple implementation.
    #[inline]
    fn into_overflow(self) -> Option<OverflowXY> {
        IntoOverflow::into_overflow((self[0], self[1]))
    }
}
