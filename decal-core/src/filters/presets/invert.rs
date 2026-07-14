use crate::filters::{
    Filter,
    primitives::TransferFunction,
};

impl Filter {
    /// Creates an invert color filter effect with the specified intensity.
    ///
    /// # Arguments
    /// - `amount`: The inversion amount where `0.0` leaves the image unchanged
    ///   and `1.0` fully inverts all colors.
    ///
    /// # Returns
    /// - [`Filter`] applying a color inversion effect.
    ///
    /// # Reference
    ///
    /// https://www.w3.org/TR/filter-effects-1/#invertEquivalent
    pub fn invert(amount: f32) -> Self {
        let a = amount.clamp(0.0, 1.0);
        let ia = 1.0 - a;

        Self::new(|ctx| {
            ctx.component_transfer()
                .func_r(TransferFunction::table(vec![a, ia]))
                .func_g(TransferFunction::table(vec![a, ia]))
                .func_b(TransferFunction::table(vec![a, ia]))
                .finish();
        })
    }
}
