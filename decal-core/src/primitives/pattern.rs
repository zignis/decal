use crate::{
    paint::ResourceIri,
    primitives::{
        IntoOptionalLength,
        Length,
        PatternContentUnits,
        PatternTransform,
        PatternUnits,
        PreserveAspectRatio,
        ViewBox,
    },
    utils::{
        ElementWriter,
        IsDefault,
    },
};
use std::fmt::{
    Display,
    Formatter,
};

type PatternUnit = Length<false, true>;

/// The SVG pattern element.
#[derive(Debug, Hash, Eq, PartialEq, Clone, Default)]
pub struct Pattern {
    content: String,
    pattern_units: PatternUnits,
    pattern_content_units: PatternContentUnits,
    x: PatternUnit,
    y: PatternUnit,
    width: PatternUnit,
    height: PatternUnit,
    preserve_aspect_ratio: PreserveAspectRatio,
    view_box: Option<ViewBox>,
    transform: PatternTransform,
}

impl Pattern {
    /// Creates a new [`Pattern`] instance from raw SVG content.
    ///
    /// # Arguments
    /// - `content`: The SVG markup rendered inside the pattern element.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new(content: String) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }

    /// Sets the coordinate system used to resolve pattern geometry.
    ///
    /// # Arguments
    /// - `value`: The [`PatternUnits`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn pattern_units<T>(mut self, value: T) -> Self
    where
        T: Into<Option<PatternUnits>>,
    {
        self.pattern_units = value.into().unwrap_or_default();
        self
    }

    /// Sets the coordinate system used to resolve pattern content.
    ///
    /// # Arguments
    /// - `value`: The [`PatternContentUnits`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn pattern_content_units<T>(mut self, value: T) -> Self
    where
        T: Into<Option<PatternContentUnits>>,
    {
        self.pattern_content_units = value.into().unwrap_or_default();
        self
    }

    /// Sets the horizontal position of the pattern tile.
    ///
    /// # Arguments
    /// - `value`: The x position value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn x<T>(mut self, value: T) -> Self
    where
        T: IntoOptionalLength<false, true>,
    {
        self.x = value.into_optional_length().unwrap_or_default();
        self
    }

    /// Sets the vertical position of the pattern tile.
    ///
    /// # Arguments
    /// - `value`: The y position value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn y<T>(mut self, value: T) -> Self
    where
        T: IntoOptionalLength<false, true>,
    {
        self.y = value.into_optional_length().unwrap_or_default();
        self
    }

    /// Sets the width of the pattern tile.
    ///
    /// # Arguments
    /// - `value`: The width value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn width<T>(mut self, value: T) -> Self
    where
        T: IntoOptionalLength<false, true>,
    {
        self.width = value.into_optional_length().unwrap_or_default();
        self
    }

    /// Sets the height of the pattern tile.
    ///
    /// # Arguments
    /// - `value`: The height value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn height<T>(mut self, value: T) -> Self
    where
        T: IntoOptionalLength<false, true>,
    {
        self.height = value.into_optional_length().unwrap_or_default();
        self
    }

    /// Sets the aspect ratio handling for the pattern view box.
    ///
    /// # Arguments
    /// - `value`: The [`PreserveAspectRatio`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn preserve_aspect_ratio<T>(mut self, value: T) -> Self
    where
        T: Into<Option<PreserveAspectRatio>>,
    {
        self.preserve_aspect_ratio = value.into().unwrap_or_default();
        self
    }

    /// Sets the view box used to map pattern content.
    ///
    /// # Arguments
    /// - `value`: The [`ViewBox`] value.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn view_box<T>(mut self, value: T) -> Self
    where
        T: Into<Option<ViewBox>>,
    {
        self.view_box = value.into();
        self
    }

    /// Applies a transformation to the pattern.
    ///
    /// # Arguments
    /// - `value`: The [`PatternTransform`] to apply.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn transform<T>(mut self, value: T) -> Self
    where
        T: Into<Option<PatternTransform>>,
    {
        self.transform = value.into().unwrap_or_default();
        self
    }
}

impl ResourceIri for Pattern {}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        ElementWriter::new(f, "pattern")?
            .attr("id", (self.iri(),))?
            .attr("viewBox", self.view_box.map(|x| (x,)))?
            .attr_if("x", self.x, !self.x.is_zero())?
            .attr_if("y", self.y, !self.y.is_zero())?
            .attr_if("width", self.width, !self.width.is_zero())?
            .attr_if("height", self.height, !self.height.is_zero())?
            .attr_if(
                "preserveAspectRatio",
                (&self.preserve_aspect_ratio,),
                !self.preserve_aspect_ratio.is_default(),
            )?
            .attr_if(
                "patternUnits",
                (&self.pattern_units,),
                !self.pattern_units.is_default(),
            )?
            .attr_if(
                "patternContentUnits",
                (&self.pattern_content_units,),
                !self.pattern_content_units.is_default(),
            )?
            .write(|out| self.transform.write(out, "patternTransform"))?
            .content(|out| out.write_str(self.content.as_str()))?
            .close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_xml;
    #[test]
    fn renders() {
        {
            let pattern = Pattern::new("content".into());
            assert_xml(
                pattern.to_string(),
                format!(r#"<pattern id="{}">content</pattern>"#, pattern.iri()),
            );
        }
    }

    #[test]
    fn renders_with_attrs() {
        let view_box = ViewBox::new(0.0, 0.0, 100.0, 150.0);
        let pattern_units = PatternUnits::UserSpaceOnUse;
        let pattern_content_units = PatternContentUnits::ObjectBoundingBox;
        let preserve_aspect_ratio = PreserveAspectRatio::new().x_min_y_min();
        let pattern = Pattern::new("<content />".into())
            .x(Length::percent(1.0))
            .y(Length::percent(2.0))
            .width(Length::percent(50.0))
            .height(Length::percent(75.0))
            .view_box(view_box)
            .pattern_units(pattern_units)
            .pattern_content_units(pattern_content_units)
            .preserve_aspect_ratio(preserve_aspect_ratio)
            .transform(PatternTransform::new().translate((1.0, 2.0)));

        assert_xml(
            pattern.to_string(),
            format!(
                r#"
<pattern
    id="{}"
    viewBox="{view_box}"
    x="1%"
    y="2%"
    width="50%"
    height="75%"
    preserveAspectRatio="{preserve_aspect_ratio}"
    patternUnits="{pattern_units}"
    patternContentUnits="{pattern_content_units}"
    patternTransform="matrix(1 0 0 1 1 2)"
>
    <content />
</pattern>
        "#,
                pattern.iri()
            ),
        );
    }
}
