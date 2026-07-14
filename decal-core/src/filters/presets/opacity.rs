use crate::filters::{
    Filter,
    primitives::TransferFunction,
};

impl Filter {
    /// Creates an opacity filter effect with the specified transparency level.
    ///
    /// # Arguments
    /// - `amount`: The opacity value where `0.0` is fully transparent and `1.0`
    ///   is fully opaque.
    ///
    /// # Returns
    /// - [`Filter`] applying an opacity adjustment effect.
    ///
    /// # Reference
    ///
    /// https://www.w3.org/TR/filter-effects-1/#opacityEquivalent
    pub fn opacity(amount: f32) -> Self {
        Self::new(|ctx| {
            ctx.component_transfer()
                .func_a(TransferFunction::table(vec![0.0, amount]))
                .finish();
        })
    }
}
