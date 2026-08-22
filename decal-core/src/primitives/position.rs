/// Specifies how a node is positioned within its parent layout.
#[derive(Debug, Copy, Clone, Default)]
pub enum Position {
    /// Positions the node relative to its computed layout position.
    #[default]
    Relative,
    /// Positions the node relative to its nearest positioned ancestor, if any.
    /// Otherwise, it is positioned relative to the origin.
    Absolute,
}

impl From<Position> for taffy::Position {
    #[inline]
    fn from(pos: Position) -> Self {
        match pos {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute,
        }
    }
}
