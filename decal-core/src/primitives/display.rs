/// Controls how a node and its children participate in layout.
#[derive(Debug, Clone, Copy, Default)]
pub enum Display {
    /// Uses the block layout algorithm.
    #[default]
    Block,
    /// Uses the flexbox layout algorithm.
    Flex,
    /// Uses the CSS Grid layout algorithm.
    #[cfg(feature = "grid")]
    Grid,
    /// Hides the node.
    None,
}

impl From<Display> for taffy::Display {
    fn from(display: Display) -> Self {
        match display {
            Display::Block => taffy::Display::Block,
            Display::Flex => taffy::Display::Flex,
            #[cfg(feature = "grid")]
            Display::Grid => taffy::Display::Grid,
            Display::None => taffy::Display::None,
        }
    }
}
