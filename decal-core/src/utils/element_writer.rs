use super::writer::Writer;
use crate::{
    primitives::{
        Length,
        PositiveF32Pair,
    },
    utils::FloatWriter,
};
use std::{
    fmt::{
        Display,
        Write,
    },
    marker::PhantomData,
};
use strict_num::{
    FiniteF32,
    NormalizedF32,
    PositiveF32,
};

pub(crate) struct Initialized;
pub(crate) struct Opened;

mod private {
    pub trait WriterState {}
}

impl private::WriterState for Initialized {}
impl private::WriterState for Opened {}

/// Stateful XML element writer with compile time enforcement of valid write
/// order.
#[derive(Debug)]
pub(crate) struct ElementWriter<'a, T, S>
where
    T: Write,
{
    out: &'a mut T,
    element_name: &'a str,
    state: PhantomData<S>,
}

impl<T, S> FloatWriter<T> for ElementWriter<'_, T, S>
where
    T: Write,
{
    fn out_mut(&mut self) -> &mut T {
        self.out
    }
}

impl<T, S> Writer<T> for ElementWriter<'_, T, S> where T: Write {}

impl<'a, T, S> ElementWriter<'a, T, S>
where
    T: Write,
    S: private::WriterState,
{
    /// Executes a write operation against the underlying output and returns the
    /// writer.
    ///
    /// # Arguments
    /// - `write_fn`: Closure that writes formatted content into the output.
    ///
    /// # Returns
    /// - `Ok(Self)` if writing succeeds.
    pub(crate) fn write<F>(self, write_fn: F) -> Result<Self, std::fmt::Error>
    where
        F: FnOnce(&mut T) -> std::fmt::Result,
    {
        write_fn(self.out).map(|_| self)
    }
}

impl<'a, T> ElementWriter<'a, T, Initialized>
where
    T: Write,
{
    /// Begins writing a new XML element with the given name.
    ///
    /// # Arguments
    /// - `out`: Output sink receiving the serialized element.
    /// - `element_name`: Name of the XML element.
    ///
    /// # Returns
    /// - [`Self`] in the [`Initialized`] state.
    pub(crate) fn new(
        out: &'a mut T,
        element_name: &'a str,
    ) -> Result<ElementWriter<'a, T, Initialized>, std::fmt::Error> {
        write!(out, "<{element_name}").map(|_| ElementWriter {
            out,
            element_name,
            state: PhantomData,
        })
    }

    /// Writes a single attribute onto the element.
    ///
    /// # Arguments
    /// - `key`: Attribute name.
    /// - `value`: Attribute value convertible via [`IntoAttrValue`].
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn attr<V>(self, key: &str, value: V) -> Result<Self, std::fmt::Error>
    where
        V: IntoAttrValue<T>,
    {
        value.write(key, self.out)?;
        Ok(self)
    }

    /// Writes multiple attributes onto the element.
    ///
    /// # Arguments
    /// - `attrs`: Iterator of key-value attribute pairs.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn attrs<'b, I, V>(self, attrs: I) -> Result<Self, std::fmt::Error>
    where
        V: IntoAttrValue<T>,
        I: IntoIterator<Item = (&'b str, V)>,
    {
        attrs
            .into_iter()
            .try_for_each(|(key, value)| value.write(key, self.out))
            .map(|_| self)
    }

    /// Conditionally writes an attribute based on a boolean flag.
    ///
    /// # Arguments
    /// - `key`: Attribute name.
    /// - `value`: Attribute value.
    /// - `condition`: Whether the attribute should be written.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn attr_if<V>(
        mut self,
        key: &str,
        value: V,
        condition: bool,
    ) -> Result<Self, std::fmt::Error>
    where
        V: IntoAttrValue<T>,
    {
        if condition {
            self = self.attr(key, value)?;
        }

        Ok(self)
    }

    /// Writes a custom attribute using a closure.
    ///
    /// # Arguments
    /// - `key`: Attribute name.
    /// - `write_fn`: Closure that writes the attribute value.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn write_attr<F>(self, key: &str, write_fn: F) -> Result<Self, std::fmt::Error>
    where
        F: FnOnce(&mut T) -> std::fmt::Result,
    {
        write!(self.out, r#" {key}=""#)?;
        write_fn(self.out)?;
        self.out.write_char('"').map(|_| self)
    }

    /// Opens the element, transitioning it to the [`Opened`] state.
    ///
    /// # Returns
    /// - [`Self`] in the [`Opened`] state.
    pub(crate) fn open(self) -> Result<ElementWriter<'a, T, Opened>, std::fmt::Error> {
        self.out.write_char('>').map(|_| ElementWriter {
            out: self.out,
            element_name: self.element_name,
            state: PhantomData,
        })
    }

    /// Writes element content without explicitly opening the element first.
    ///
    /// # Arguments
    /// - `write_fn`: Closure that writes element content.
    ///
    /// # Returns
    /// - [`Self`] in the [`Opened`] state.
    pub(crate) fn content<F>(
        self,
        write_fn: F,
    ) -> Result<ElementWriter<'a, T, Opened>, std::fmt::Error>
    where
        F: FnOnce(&mut T) -> std::fmt::Result,
    {
        let writer = self.open()?;
        writer.content(write_fn)
    }

    /// Closes the element as a self-closing tag.
    pub(crate) fn close(self) -> std::fmt::Result {
        self.out.write_str(" />")
    }

    /// Writes a closing tag for the given element name.
    ///
    /// # Arguments
    /// - `out`: The output writer.
    /// - `element_name`: Name of the element to close.
    pub(crate) fn close_tag(out: &'a mut T, element_name: &str) -> std::fmt::Result {
        write!(out, "</{element_name}>")
    }
}

impl<'a, T> ElementWriter<'a, T, Opened>
where
    T: Write,
{
    /// Writes content inside an opened element.
    ///
    /// # Arguments
    /// - `write_fn`: Closure that writes content.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn content<F>(self, write_fn: F) -> Result<Self, std::fmt::Error>
    where
        F: FnOnce(&mut T) -> std::fmt::Result,
    {
        self.write(write_fn)
    }

    /// Closes the element with a matching end tag.
    pub(crate) fn close(self) -> std::fmt::Result {
        write!(self.out, "</{}>", self.element_name)
    }
}

/// Trait for types that can be written as XML attribute values.
pub(crate) trait IntoAttrValue<W>
where
    W: Write,
{
    /// Writes the value as an attribute with the given key.
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result;
}

impl<'a, W, D> IntoAttrValue<W> for (D,)
where
    W: Write,
    D: Display,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        write!(out, r#" {key}="{}""#, self.0)
    }
}

impl<W> IntoAttrValue<W> for &str
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        write!(out, r#" {key}="{self}""#)
    }
}

impl<'a, W> IntoAttrValue<W> for String
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        IntoAttrValue::write(&self.as_str(), key, out)
    }
}

impl<'a, W> IntoAttrValue<W> for f32
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        write!(out, r#" {key}=""#)?;
        out.write_float(*self)?;
        out.write_char('"')
    }
}

impl<'a, W, const AUTO: bool, const PERCENT: bool> IntoAttrValue<W> for Length<AUTO, PERCENT>
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        IntoAttrValue::write(&(self,), key, out)
    }
}

impl<'a, W> IntoAttrValue<W> for FiniteF32
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        IntoAttrValue::write(&self.get(), key, out)
    }
}

impl<'a, W> IntoAttrValue<W> for NormalizedF32
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        IntoAttrValue::write(&self.get(), key, out)
    }
}

impl<'a, W> IntoAttrValue<W> for PositiveF32
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        IntoAttrValue::write(&self.get(), key, out)
    }
}

impl<'a, W> IntoAttrValue<W> for PositiveF32Pair
where
    W: Write,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        write!(out, r#" {key}="{self}""#)
    }
}

impl<'a, W, V> IntoAttrValue<W> for Option<V>
where
    W: Write,
    V: IntoAttrValue<W>,
{
    fn write(&self, key: &str, out: &mut W) -> std::fmt::Result {
        if let Some(value) = self {
            IntoAttrValue::write(value, key, out)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        macros::{
            ff32,
            nf32,
            pf32,
        },
        test_utils::{
            assert_xml,
            str_sink,
        },
    };

    #[test]
    fn writes_empty_element() {
        assert_xml(
            str_sink(|out| ElementWriter::new(out, "path")?.close()),
            r#"<path />"#,
        );
    }

    #[test]
    fn writes_single_attr() {
        assert_xml(
            str_sink(|out| ElementWriter::new(out, "rect")?.attr("id", "test")?.close()),
            r#"<rect id="test" />"#,
        );
    }

    #[test]
    fn writes_multiple_attrs() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "rect")?
                    .attrs([("x", 10.0), ("y", 25.0), ("width", 96.0), ("height", 64.0)])?
                    .close()
            }),
            r#"<rect x="10" y="25" width="96" height="64" />"#,
        );
    }

    #[test]
    fn attr_if_writes_conditionally() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "circle")?
                    .attr_if("cx", 50.0, true)?
                    .attr_if("cy", 50.0, false)?
                    .close()
            }),
            r#"<circle cx="50" />"#,
        );
    }

    #[test]
    fn element_with_content() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "text")?
                    .open()?
                    .content(|out| out.write_str("hello"))?
                    .close()
            }),
            r#"<text>hello</text>"#,
        );
    }

    #[test]
    fn content_without_manually_opening() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "text")?
                    .content(|out| out.write_str("hello"))?
                    .close()
            }),
            r#"<text>hello</text>"#,
        );
    }

    #[test]
    fn custom_attr_writer() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "polygon")?
                    .write_attr("points", |out| out.write_str("0,0 10,10"))?
                    .close()
            }),
            r#"<polygon points="0,0 10,10" />"#,
        );
    }

    #[test]
    fn writes_float_attrs() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "group")?
                    .attr("x", nf32!(0.5))?
                    .attr("y", pf32!(1.5))?
                    .attr("z", ff32!(2.5))?
                    .close()
            }),
            r#"<group x="0.5" y="1.5" z="2.5" />"#,
        );
    }

    #[test]
    fn writes_length_attr() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "rect")?
                    .attr("width", Length::<false, false>::units(100.0))?
                    .close()
            }),
            r#"<rect width="100" />"#,
        );
    }

    #[test]
    fn writes_positive_f32_pair() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "feMorphology")?
                    .attr("radius", PositiveF32Pair::from((1.5, 2.5)))?
                    .close()
            }),
            r#"<feMorphology radius="1.5 2.5" />"#,
        );
    }

    #[test]
    fn writes_optional_attr() {
        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "rect")?
                    .attr("x", Some(10.0))?
                    .close()
            }),
            r#"<rect x="10" />"#,
        );

        assert_xml(
            str_sink(|out| {
                ElementWriter::new(out, "rect")?
                    .attr("x", None::<f32>)?
                    .close()
            }),
            "<rect />",
        );
    }

    #[test]
    fn writes_close_tag() {
        assert_eq!(str_sink(|out| ElementWriter::close_tag(out, "g")), "</g>");
    }
}
