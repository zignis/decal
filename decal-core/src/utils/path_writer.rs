use super::writer::Writer;
use crate::utils::FloatWriter;
use std::fmt::Write;

/// Utility writer for constructing SVG path data commands.
#[derive(Debug)]
pub(crate) struct PathWriter<'a, T>(&'a mut T)
where
    T: Write;

impl<T> FloatWriter<T> for PathWriter<'_, T>
where
    T: Write,
{
    fn out_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<T> Writer<T> for PathWriter<'_, T> where T: Write {}

impl<'a, T> PathWriter<'a, T>
where
    T: Write,
{
    /// Creates a new [`PathWriter`] instance.
    ///
    /// # Arguments
    /// - `out`: The output sink receiving path commands.
    ///
    /// # Returns
    /// - [`Self`]
    pub(crate) fn new(out: &'a mut T) -> Self {
        Self(out)
    }

    /// Writes an absolute move-to command.
    ///
    /// # Arguments
    /// - `x`: X coordinate of the target point.
    /// - `y`: Y coordinate of the target point.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn move_to(&mut self, x: f32, y: f32) -> Result<&mut Self, std::fmt::Error> {
        self.char('M')?.float(x)?.space()?.float(y)?.space()
    }

    /// Writes an absolute line-to command.
    ///
    /// # Arguments
    /// - `x`: X coordinate of the target point.
    /// - `y`: Y coordinate of the target point.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn line_to(&mut self, x: f32, y: f32) -> Result<&mut Self, std::fmt::Error> {
        self.char('L')?.float(x)?.space()?.float(y)?.space()
    }

    /// Writes an absolute horizontal line command.
    ///
    /// # Arguments
    /// - `x`: Target X coordinate.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn horizontal_to(&mut self, x: f32) -> Result<&mut Self, std::fmt::Error> {
        self.char('H')?.float(x)?.space()
    }

    /// Writes an absolute vertical line command.
    ///
    /// # Arguments
    /// - `y`: Target Y coordinate.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn vertical_to(&mut self, y: f32) -> Result<&mut Self, std::fmt::Error> {
        self.char('V')?.float(y)?.space()
    }

    /// Writes an absolute quadratic Bézier curve command.
    ///
    /// # Arguments
    /// - `cx`: X coordinate of the control point.
    /// - `cy`: Y coordinate of the control point.
    /// - `x`: X coordinate of the end point.
    /// - `y`: Y coordinate of the end point.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn quad_to(
        &mut self,
        cx: f32,
        cy: f32,
        x: f32,
        y: f32,
    ) -> Result<&mut Self, std::fmt::Error> {
        self.char('Q')?
            .float(cx)?
            .space()?
            .float(cy)?
            .space()?
            .float(x)?
            .space()?
            .float(y)?
            .space()
    }

    /// Writes an absolute cubic Bézier curve command.
    ///
    /// # Arguments
    /// - `cx1`: X coordinate of the first control point.
    /// - `cy1`: Y coordinate of the first control point.
    /// - `cx2`: X coordinate of the second control point.
    /// - `cy2`: Y coordinate of the second control point.
    /// - `x`: X coordinate of the end point.
    /// - `y`: Y coordinate of the end point.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn curve_to(
        &mut self,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> Result<&mut Self, std::fmt::Error> {
        self.char('C')?
            .float(cx1)?
            .space()?
            .float(cy1)?
            .space()?
            .float(cx2)?
            .space()?
            .float(cy2)?
            .space()?
            .float(x)?
            .space()?
            .float(y)?
            .space()
    }

    /// Writes an absolute arc command with fixed flags.
    ///
    /// # Arguments
    /// - `rx`: X axis radius.
    /// - `ry`: Y axis radius.
    /// - `x`: X coordinate of the end point.
    /// - `y`: Y coordinate of the end point.
    ///
    /// # Returns
    /// - `Ok(&mut Self)` for chaining.
    pub(crate) fn arc_to(
        &mut self,
        rx: f32,
        ry: f32,
        x: f32,
        y: f32,
    ) -> Result<&mut Self, std::fmt::Error> {
        self.char('A')?
            .float(rx)?
            .space()?
            .float(ry)?
            .str(" 0 0 1 ")?
            .float(x)?
            .space()?
            .float(y)?
            .space()
    }

    /// Writes a close-path command.
    pub(crate) fn close(&mut self) -> std::fmt::Result {
        self.char('Z').map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_path<F>(write_fn: F) -> String
    where
        F: FnOnce(&mut PathWriter<'_, String>) -> std::fmt::Result,
    {
        let mut out = String::new();
        let mut d = PathWriter::new(&mut out);
        write_fn(&mut d).unwrap();
        out
    }

    #[test]
    fn writes_move_to() {
        assert_eq!(write_path(|d| d.move_to(10.0, 15.0).map(|_| ())), "M10 15 ");
    }

    #[test]
    fn writes_line_to() {
        assert_eq!(write_path(|d| d.line_to(2.5, 3.5).map(|_| ())), "L2.5 3.5 ");
    }

    #[test]
    fn writes_horizontal_to() {
        assert_eq!(
            write_path(|d| d.horizontal_to(15.75).map(|_| ())),
            "H15.75 "
        );
    }

    #[test]
    fn writes_vertical_to() {
        assert_eq!(write_path(|d| d.vertical_to(15.75).map(|_| ())), "V15.75 ");
    }

    #[test]
    fn writes_quad_to() {
        assert_eq!(
            write_path(|d| d.quad_to(1.0, 2.0, 3.0, 4.0).map(|_| ())),
            "Q1 2 3 4 "
        );
    }

    #[test]
    fn writes_curve_to() {
        assert_eq!(
            write_path(|d| d.curve_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0).map(|_| ())),
            "C1 2 3 4 5 6 "
        );
    }

    #[test]
    fn writes_arc_to() {
        assert_eq!(
            write_path(|d| d.arc_to(1.0, 2.0, 3.0, 4.0).map(|_| ())),
            "A1 2 0 0 1 3 4 "
        );
    }

    #[test]
    fn close_without_commands() {
        assert_eq!(write_path(|d| d.close()), "Z");
    }

    #[test]
    fn closes_with_commands() {
        assert_eq!(
            write_path(|d| d.move_to(0.0, 0.0)?.line_to(10.0, 0.0)?.close()),
            "M0 0 L10 0 Z"
        );
    }

    #[test]
    fn rounds_float_values() {
        assert_eq!(
            write_path(|d| d.move_to(1.23456, 2.000004).map(|_| ())),
            "M1.2346 2 "
        );
    }
}
