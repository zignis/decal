use crate::{
    filters::{
        FilterRegion,
        FilterRegionConfig,
        HasFilterRegion,
        context::FilterContext,
        primitives::FilterPrimitive,
    },
    paint::ResourceIri,
    primitives::{
        ColorInterpolation,
        FilterUnits,
        PrimitiveUnits,
    },
    utils::{
        ElementWriter,
        IsDefault,
    },
};
use smart_default::SmartDefault;
use std::{
    fmt::{
        Display,
        Formatter,
    },
    hash::Hash,
};

/// The container for one or more [`FilterPrimitive`].
#[derive(Debug, Hash, Eq, PartialEq, Clone, SmartDefault)]
pub struct Filter {
    filter_units: FilterUnits,
    primitive_units: PrimitiveUnits,
    primitives: Vec<FilterPrimitive>,
    region: FilterRegion,
    #[default(ColorInterpolation::LinearRgb)]
    color_interpolation: ColorInterpolation,
}

impl Filter {
    /// Creates a new [`Filter`] by executing a builder function on a fresh
    /// [`FilterContext`].
    ///
    /// The provided closure receives a mutable reference to a
    /// [`FilterContext`], which is used to construct and configure filter
    /// primitives.
    ///
    /// # Note
    /// Primitives are deduplicated automatically by the context, so
    /// extra care is needed when defining multiple instances of similar filter
    /// primitives, expecting them to use the result of the previous filter
    /// primitive as their input.
    ///
    /// # Arguments
    /// - `build`: The closure used to populate the filter with primitives.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use decal::prelude::*;
    ///
    /// let filter = Filter::new(|ctx| {
    ///     let background = ctx.flood().color(Color::rgb(255, 255, 0)).finish();
    ///
    ///     ctx.gaussian_blur()
    ///         .input(background)
    ///         .std_deviation(5.0)
    ///         .finish();
    /// });
    /// ```
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new<T>(build: T) -> Self
    where
        T: FnOnce(&mut FilterContext),
    {
        Filter {
            primitives: {
                let mut ctx = FilterContext::default();
                build(&mut ctx);
                ctx.into_primitives()
            },
            ..Default::default()
        }
    }

    /// Sets the coordinate system used for interpreting the filter region.
    ///
    /// # Arguments
    /// - `value`: The [`FilterUnits`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn filter_units<I>(mut self, value: I) -> Self
    where
        I: Into<Option<FilterUnits>>,
    {
        self.filter_units = value.into().unwrap_or_default();
        self
    }

    /// Sets the coordinate system used for interpreting filter primitive
    /// subregions.
    ///
    /// # Arguments
    /// - `value`: The [`PrimitiveUnits`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn primitive_units<I>(mut self, value: I) -> Self
    where
        I: Into<Option<PrimitiveUnits>>,
    {
        self.primitive_units = value.into().unwrap_or_default();
        self
    }

    /// Sets the color interpolation space used during filter evaluation.
    ///
    /// # Arguments
    /// - `value`: The [`ColorInterpolation`] space to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn color_interpolation(mut self, value: ColorInterpolation) -> Self {
        self.color_interpolation = value;
        self
    }

    /// Appends another filter into this filter. Metadata is overwritten by the
    /// appended filter.
    fn append(mut self, next: Filter) -> Self {
        self.filter_units = next.filter_units;
        self.primitive_units = next.primitive_units;
        self.region = next.region;
        self.color_interpolation = next.color_interpolation;
        self.primitives.extend(next.primitives);

        self
    }

    /// Returns the URL of all image filter primitives
    pub(crate) fn image_refs(&self) -> Vec<String> {
        let mut image_refs = Vec::new();

        for primitive in &self.primitives {
            if let FilterPrimitive::Image(image) = primitive {
                image_refs.push(image.href().to_string());
            }
        }

        image_refs
    }
}

impl HasFilterRegion for Filter {
    fn region_mut(&mut self) -> &mut FilterRegion {
        &mut self.region
    }
}

impl FilterRegionConfig for Filter {}
impl IsDefault for Filter {}
impl ResourceIri for Filter {}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        ElementWriter::new(f, "filter")?
            .write(|out| self.region.fmt(out))?
            .attr("id", (self.iri(),))?
            .attr_if(
                "filterUnits",
                (&self.filter_units,),
                !self.filter_units.is_default(),
            )?
            .attr_if(
                "primitiveUnits",
                (&self.primitive_units,),
                !self.primitive_units.is_default(),
            )?
            .attr_if(
                "color-interpolation-filters",
                (&self.color_interpolation,),
                self.color_interpolation != ColorInterpolation::LinearRgb,
            )?
            .content(|out| {
                self.primitives
                    .iter()
                    .try_for_each(|primitive| primitive.fmt(out))
            })?
            .close()
    }
}

//

impl From<Vec<Filter>> for Filter {
    #[inline]
    fn from(value: Vec<Filter>) -> Self {
        value
            .into_iter()
            .fold(Filter::default(), |acc, next| acc.append(next))
    }
}

impl<const N: usize> From<[Filter; N]> for Filter {
    #[inline]
    fn from(value: [Filter; N]) -> Self {
        value
            .into_iter()
            .fold(Filter::default(), |acc, next| acc.append(next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_xml;

    #[test]
    fn renders_with_no_primitives() {
        let filter = Filter::new(|_| {});
        assert_xml(
            filter.to_string(),
            format!(r#"<filter id="{}"></filter>"#, filter.iri()),
        );
    }

    #[test]
    fn renders_with_filter_region() {
        let filter = Filter::new(|_| {}).x(0.5).y(0.6).width(110).height(120);
        assert_xml(
            filter.to_string(),
            format!(
                r#"<filter id="{}" x="0.5" y="0.6" width="110" height="120"></filter>"#,
                filter.iri()
            ),
        );
    }

    #[test]
    fn renders() {
        let filter = Filter::new(|ctx| {
            ctx.flood().finish();
            ctx.gaussian_blur().finish();
        });
        let flood = &filter.primitives[0];
        let blur = &filter.primitives[1];

        assert_xml(
            filter.to_string(),
            format!(
                r#"
<filter id="{}">
    {flood}
    {blur}
</filter>
"#,
                filter.iri()
            ),
        );
    }

    #[test]
    fn renders_with_attrs() {
        let filter_units = FilterUnits::UserSpaceOnUse;
        let primitive_units = PrimitiveUnits::ObjectBoundingBox;
        let color_interpolation = ColorInterpolation::SRgb;
        let filter = Filter::new(|_| {})
            .filter_units(filter_units)
            .primitive_units(primitive_units)
            .color_interpolation(color_interpolation);

        assert_xml(
            filter.to_string(),
            format!(
                r#"
<filter
    id="{}"
    filterUnits="{filter_units}"
    primitiveUnits="{primitive_units}"
    color-interpolation-filters="{color_interpolation}">
</filter>
"#,
                filter.iri()
            ),
        );
    }

    #[test]
    fn appends_filter() {
        let filter = Filter::from(vec![
            Filter::new(|ctx| {
                ctx.flood().finish();
            }),
            Filter::new(|ctx| {
                ctx.gaussian_blur().finish();
            }),
            Filter::new(|ctx| {
                ctx.flood().finish();
            }),
        ]);
        let primitives = &filter.primitives;

        assert_xml(
            filter.to_string(),
            format!(
                r#"
<filter id="{}">
    {}{}{}
</filter>
"#,
                filter.iri(),
                primitives[0],
                primitives[1],
                primitives[2]
            ),
        );
    }

    #[test]
    fn overwrites_metadata_from_last_filter() {
        let filter = Filter::from([Filter::new(|_| {}).x(-45), Filter::new(|_| {}).x(-30)]);

        assert_xml(
            filter.to_string(),
            format!(
                r#"
<filter id="{}" x="-30"></filter>
"#,
                filter.iri()
            ),
        );
    }
}
