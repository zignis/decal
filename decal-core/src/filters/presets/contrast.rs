use crate::filters::{
    Filter,
    primitives::TransferFunction,
};

impl Filter {
    /// Creates a contrast filter effect with the specified multiplier.
    ///
    /// # Arguments
    /// - `amount`: The contrast multiplier applied to each color channel.
    ///
    /// # Returns
    /// - [`Filter`] applying a contrast adjustment effect.
    ///
    /// # Reference
    ///
    /// https://www.w3.org/TR/filter-effects-1/#contrastEquivalent
    pub fn contrast(amount: f32) -> Self {
        let intercept = -(0.5 * amount) + 0.5;

        Self::new(|ctx| {
            ctx.component_transfer()
                .func_r(TransferFunction::linear(amount, intercept))
                .func_g(TransferFunction::linear(amount, intercept))
                .func_b(TransferFunction::linear(amount, intercept))
                .finish();
        })
    }
}
