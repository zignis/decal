use crate::{
    filters::primitives::{
        Blend,
        ColorMatrix,
        ComponentTransfer,
        Composite,
        ConvolveMatrix,
        DiffuseLighting,
        DisplacementMap,
        DropShadow,
        FilterPrimitive,
        Flood,
        GaussianBlur,
        Image,
        Merge,
        Morphology,
        Offset,
        PrimitiveBuilder,
        SpecularLighting,
        Tile,
        Turbulence,
    },
    paint::{
        Iri,
        ResourceIri,
    },
    primitives::LightSource,
};
use hashbrown::HashMap;
use parking_lot::Mutex;

/// The handle to a finalized filter primitive stored in a [`FilterContext`].
/// Can be used as a filter input for other filter primitive nodes.
#[derive(Debug, Copy, Clone)]
pub struct PrimitiveNode {
    #[allow(dead_code)]
    idx: usize,
    iri: Iri,
}

impl PrimitiveNode {
    pub(crate) fn iri(&self) -> Iri {
        self.iri
    }
}

#[derive(Debug, Default)]
struct ContextInner {
    index_map: HashMap<FilterPrimitive, usize>,
    primitives: Vec<FilterPrimitive>,
}

/// The context for building and deduplicating filter primitives.
#[derive(Debug, Default)]
pub struct FilterContext(Mutex<ContextInner>);

impl FilterContext {
    /// Creates a new [`Flood`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn flood(&self) -> PrimitiveBuilder<'_, Flood> {
        PrimitiveBuilder::new(self, Flood::new())
    }

    /// Creates a new [`Image`] filter primitive.
    ///
    /// # Arguments
    /// - `href`: The image source.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn image(&self, href: &str) -> PrimitiveBuilder<'_, Image> {
        PrimitiveBuilder::new(self, Image::new(href))
    }

    /// Creates a new [`GaussianBlur`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn gaussian_blur(&self) -> PrimitiveBuilder<'_, GaussianBlur> {
        PrimitiveBuilder::new(self, GaussianBlur::new())
    }

    /// Creates a new [`Turbulence`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn turbulence(&self) -> PrimitiveBuilder<'_, Turbulence> {
        PrimitiveBuilder::new(self, Turbulence::new())
    }

    /// Creates a new [`ColorMatrix`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn color_matrix(&self) -> PrimitiveBuilder<'_, ColorMatrix> {
        PrimitiveBuilder::new(self, ColorMatrix::new())
    }

    /// Creates a new [`ComponentTransfer`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn component_transfer(&self) -> PrimitiveBuilder<'_, ComponentTransfer> {
        PrimitiveBuilder::new(self, ComponentTransfer::new())
    }

    /// Creates a new [`DisplacementMap`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn displacement_map(&self) -> PrimitiveBuilder<'_, DisplacementMap> {
        PrimitiveBuilder::new(self, DisplacementMap::new())
    }

    /// Creates a new [`Composite`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn composite(&self) -> PrimitiveBuilder<'_, Composite> {
        PrimitiveBuilder::new(self, Composite::new())
    }

    /// Creates a new [`Blend`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn blend(&self) -> PrimitiveBuilder<'_, Blend> {
        PrimitiveBuilder::new(self, Blend::new())
    }

    /// Creates a new [`Merge`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn merge(&self) -> PrimitiveBuilder<'_, Merge> {
        PrimitiveBuilder::new(self, Merge::new())
    }

    /// Creates a new [`SpecularLighting`] filter primitive.
    ///
    /// # Arguments
    /// - `light_source`: The [`LightSource`] used for lighting.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn specular_lighting(
        &self,
        light_source: LightSource,
    ) -> PrimitiveBuilder<'_, SpecularLighting> {
        PrimitiveBuilder::new(self, SpecularLighting::new(light_source))
    }

    /// Creates a new [`DiffuseLighting`] filter primitive.
    ///
    /// # Arguments
    /// - `light_source`: The [`LightSource`] used for lighting.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn diffuse_lighting(
        &self,
        light_source: LightSource,
    ) -> PrimitiveBuilder<'_, DiffuseLighting> {
        PrimitiveBuilder::new(self, DiffuseLighting::new(light_source))
    }

    /// Creates a new [`ConvolveMatrix`] filter primitive.
    ///
    /// # Arguments
    /// - `kernel_matrix`: The convolution kernel values.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn convolve_matrix(&self, kernel_matrix: Vec<f32>) -> PrimitiveBuilder<'_, ConvolveMatrix> {
        PrimitiveBuilder::new(self, ConvolveMatrix::new(kernel_matrix))
    }

    /// Creates a new [`DropShadow`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn drop_shadow(&self) -> PrimitiveBuilder<'_, DropShadow> {
        PrimitiveBuilder::new(self, DropShadow::new())
    }

    /// Creates a new [`Morphology`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn morphology(&self) -> PrimitiveBuilder<'_, Morphology> {
        PrimitiveBuilder::new(self, Morphology::new())
    }

    /// Creates a new [`Offset`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn offset(&self) -> PrimitiveBuilder<'_, Offset> {
        PrimitiveBuilder::new(self, Offset::new())
    }

    /// Creates a new [`Tile`] filter primitive.
    ///
    /// # Returns
    /// - [`PrimitiveBuilder`]
    pub fn tile(&self) -> PrimitiveBuilder<'_, Tile> {
        PrimitiveBuilder::new(self, Tile::new())
    }

    /// Consumes the context and returns all collected filter primitives.
    pub(crate) fn into_primitives(self) -> Vec<FilterPrimitive> {
        self.0.into_inner().primitives
    }

    /// Inserts a filter primitive into the context or returns an existing one
    /// if it is already present.
    pub(crate) fn get_or_add_primitive(&self, primitive: FilterPrimitive) -> PrimitiveNode {
        let mut inner = self.0.lock();
        let iri = primitive.iri();

        if let Some(idx) = inner.index_map.get(&primitive) {
            return PrimitiveNode { idx: *idx, iri };
        }

        let idx = inner.primitives.len();
        inner.index_map.insert(primitive.clone(), idx);
        inner.primitives.push(primitive);

        PrimitiveNode { idx, iri }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_empty_context() {
        assert!(FilterContext::default().into_primitives().is_empty());
    }

    #[test]
    fn primitives_are_deduplicated() {
        let ctx = FilterContext::default();

        ctx.flood().finish();
        ctx.flood().finish();

        let primitives = ctx.into_primitives();
        assert_eq!(primitives.len(), 1);
    }

    #[test]
    fn adds_multiple_primitives() {
        let ctx = FilterContext::default();

        ctx.gaussian_blur().finish();
        ctx.flood().finish();

        let primitives = ctx.into_primitives();
        assert_eq!(primitives.len(), 2);

        //

        let ctx = FilterContext::default();

        ctx.gaussian_blur().std_deviation(5.0).finish();
        ctx.gaussian_blur().std_deviation(2.0).finish();
        ctx.gaussian_blur()
            .input(ctx.gaussian_blur().finish())
            .finish();

        let primitives = ctx.into_primitives();
        assert_eq!(primitives.len(), 4);
    }
}
