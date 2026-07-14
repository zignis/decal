use crate::filters::{
    Filter,
    primitives::TransferFunction,
};

const MIN_LEVELS: u8 = 2;
const MAX_LEVELS: u8 = 255;

impl Filter {
    /// Creates a poster filter effect by reducing the number of discrete color
    /// levels per RGB channel.
    ///
    /// # Arguments
    /// - `levels`: Number of discrete color levels per channel.
    ///
    /// # Returns
    /// - [`Filter`] applying a posterization effect.
    pub fn posterize(levels: u8) -> Self {
        let levels = levels.clamp(MIN_LEVELS, MAX_LEVELS);
        let mut values = Vec::with_capacity(levels as usize);

        for i in 0..levels {
            let value = (i as f32) / (levels - 1) as f32;
            values.push(value.clamp(0.0, 1.0));
        }

        Self::new(move |ctx| {
            ctx.component_transfer()
                .func_r(TransferFunction::discrete(values.clone()))
                .func_g(TransferFunction::discrete(values.clone()))
                .func_b(TransferFunction::discrete(values.clone()))
                .finish();
        })
    }
}
