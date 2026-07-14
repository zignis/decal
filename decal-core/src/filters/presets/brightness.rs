use crate::filters::{
    Filter,
    primitives::TransferFunction,
};

impl Filter {
    /// Creates a brightness filter effect with the specified multiplier.
    ///
    /// # Arguments
    /// - `amount`: The brightness multiplier applied to each color channel.
    ///
    /// # Returns
    /// - [`Filter`] applying a brightness adjustment effect.
    ///
    /// # Reference
    ///
    /// https://www.w3.org/TR/filter-effects-1/#brightnessEquivalent
    pub fn brightness(amount: f32) -> Self {
        Self::new(|ctx| {
            ctx.component_transfer()
                .func_r(TransferFunction::linear(amount, 0.0))
                .func_g(TransferFunction::linear(amount, 0.0))
                .func_b(TransferFunction::linear(amount, 0.0))
                .finish();
        })
    }
}
