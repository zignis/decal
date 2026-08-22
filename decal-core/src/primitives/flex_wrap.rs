/// Controls whether flex wrap behavior.
#[derive(Debug, Copy, Clone)]
pub enum FlexWrap {
    /// Items will not wrap, even if they overflow the container.
    NoWrap,
    /// Allows items to wrap.
    Wrap,
    /// Allows items to wrap in the reverse direction.
    WrapReverse,
}

impl From<FlexWrap> for taffy::FlexWrap {
    #[inline]
    fn from(wrap: FlexWrap) -> Self {
        match wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }
}
