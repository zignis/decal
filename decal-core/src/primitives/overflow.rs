/// Controls how content that overflows the bounds of a node is handled.
#[derive(Debug, Clone, Copy, Default)]
pub enum Overflow {
    /// Overflowing content is visible and may extend outside the node's bounds.
    #[default]
    Visible,
    /// Overflowing content is clipped to the node's bounds.
    Hidden,
}

impl From<Overflow> for taffy::Overflow {
    fn from(overflow: Overflow) -> Self {
        match overflow {
            Overflow::Visible => taffy::Overflow::Visible,
            Overflow::Hidden => taffy::Overflow::Hidden,
        }
    }
}
