use crate::utils::FloatWriter;
use std::fmt::Write;
use usvg::tiny_skia_path;

#[derive(Debug, Clone)]
enum RotationAnchor {
    Center,          // rotate around object's center
    Origin,          // rotate around origin of system coords
    Point(f32, f32), // rotate around a point
}

#[derive(Debug, Clone)]
enum TransformOperation {
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32, RotationAnchor),
    Skew(f32, f32),
}

/// The node transformation.
#[derive(Debug, Clone, Default)]
pub struct Transform {
    initial_tf: Option<usvg::Transform>,
    operations: Vec<TransformOperation>,
}

impl Transform {
    /// Creates a new identity [`Transform`] instance.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Transform`] from the provided matrix values.
    ///
    /// # Arguments
    /// - `sx`: The horizontal scale component.
    /// - `ky`: The vertical skew component.
    /// - `kx`: The horizontal skew component.
    /// - `sy`: The vertical scale component.
    /// - `tx`: The horizontal translation component.
    /// - `ty`: The vertical translation component.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn matrix(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Transform {
            initial_tf: Some(usvg::Transform::from_row(sx, ky, kx, sy, tx, ty)),
            ..Default::default()
        }
    }

    /// Translates the node along both axes.
    ///
    /// # Arguments
    /// - `value`: The translation distance convertible using [`IntoFloatPair`].
    ///
    /// # Returns
    /// - [`Self`]
    pub fn translate<T>(mut self, value: T) -> Self
    where
        T: IntoFloatPair,
    {
        let (x, y) = value.into_float_pair();
        self.operations.push(TransformOperation::Translate(x, y));
        self
    }

    /// Translates the node along the x-axis.
    ///
    /// # Arguments
    /// - `value`: The translation distance along the x-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn translate_x(mut self, value: f32) -> Self {
        self.operations
            .push(TransformOperation::Translate(value, 0.0));
        self
    }

    /// Translates the node along the y-axis.
    ///
    /// # Arguments
    /// - `value`: The translation distance along the y-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn translate_y(mut self, value: f32) -> Self {
        self.operations
            .push(TransformOperation::Translate(0.0, value));
        self
    }

    /// Scales the node along both axes.
    ///
    /// # Arguments
    /// - `value`: The scaling factor convertible using [`IntoFloatPair`].
    ///
    /// # Returns
    /// - [`Self`]
    pub fn scale<T>(mut self, value: T) -> Self
    where
        T: IntoFloatPair,
    {
        let (x, y) = value.into_float_pair();
        self.operations.push(TransformOperation::Scale(x, y));
        self
    }

    /// Scales the node along the x-axis.
    ///
    /// # Arguments
    /// - `value`: The scaling factor along the x-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn scale_x(mut self, value: f32) -> Self {
        self.operations.push(TransformOperation::Scale(value, 1.0));
        self
    }

    /// Scales the node along the y-axis.
    ///
    /// # Arguments
    /// - `value`: The scaling factor along the y-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn scale_y(mut self, value: f32) -> Self {
        self.operations.push(TransformOperation::Scale(1.0, value));
        self
    }

    /// Rotates the node around its center.
    ///
    /// # Arguments
    /// - `angle`: The rotation angle in degrees.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn rotate(mut self, angle: f32) -> Self {
        self.operations
            .push(TransformOperation::Rotate(angle, RotationAnchor::Center));
        self
    }

    /// Rotates the node around the coordinate origin.
    ///
    /// # Arguments
    /// - `angle`: The rotation angle in degrees.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn rotate_origin(mut self, angle: f32) -> Self {
        self.operations
            .push(TransformOperation::Rotate(angle, RotationAnchor::Origin));
        self
    }

    /// Rotates the node around a specific point.
    ///
    /// # Arguments
    /// - `angle`: The rotation angle in degrees.
    /// - `x`: The x coordinate of the rotation anchor.
    /// - `y`: The y coordinate of the rotation anchor.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn rotate_at(mut self, angle: f32, x: f32, y: f32) -> Self {
        self.operations.push(TransformOperation::Rotate(
            angle,
            RotationAnchor::Point(x, y),
        ));
        self
    }

    /// Skews the node along both axes.
    ///
    /// # Arguments
    /// - `value`: The skew angle convertible using [`IntoFloatPair`].
    ///
    /// # Returns
    /// - [`Self`]
    pub fn skew<T>(mut self, value: T) -> Self
    where
        T: IntoFloatPair,
    {
        let (x, y) = value.into_float_pair();
        self.operations.push(TransformOperation::Skew(x, y));
        self
    }

    /// Skews the node along the x-axis.
    ///
    /// # Arguments
    /// - `value`: The skew angle along the x-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn skew_x(mut self, angle: f32) -> Self {
        self.operations.push(TransformOperation::Skew(angle, 0.0));
        self
    }

    /// Skews the node along the y-axis.
    ///
    /// # Arguments
    /// - `value`: The skew angle along the y-axis.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn skew_y(mut self, angle: f32) -> Self {
        self.operations.push(TransformOperation::Skew(0.0, angle));
        self
    }

    /// Writes the resolved transform as an SVG matrix attribute.
    ///
    /// # Arguments
    /// - `out`: The output writer.
    /// - `pos`: The top-left position of the node.
    /// - `translate`: Translation applied before local transforms.
    /// - `size`: The size of the node used for center-based transforms.
    pub(crate) fn write<T>(
        &self,
        out: &mut T,
        pos: (f32, f32),
        translate: (f32, f32),
        size: (f32, f32),
    ) -> std::fmt::Result
    where
        T: Write,
    {
        let mut tf = self
            .initial_tf
            .unwrap_or_default()
            .post_translate(translate.0, translate.1);

        let center = tiny_skia_path::Point {
            x: pos.0 + size.0 / 2.0,
            y: pos.1 + size.1 / 2.0,
        };

        for op in &self.operations {
            // center of obj before ops
            let mut before = center;
            tf.map_point(&mut before);
            let mut recenter = true;

            match *op {
                TransformOperation::Translate(x, y) => {
                    tf = tf.post_translate(x, y);
                    recenter = false;
                }

                TransformOperation::Scale(x, y) => {
                    tf = tf.post_scale(x, y);
                }

                TransformOperation::Rotate(angle, ref anchor) => match anchor {
                    RotationAnchor::Center => {
                        tf = tf.post_rotate(angle);
                    }
                    RotationAnchor::Origin => {
                        tf = tf.post_rotate(angle);
                        recenter = false;
                    }
                    RotationAnchor::Point(x, y) => {
                        tf = tf.post_rotate_at(angle, *x, *y);
                        recenter = false;
                    }
                },

                TransformOperation::Skew(x, y) => {
                    tf = tf.post_concat(usvg::Transform::from_skew(x, y));
                }
            }

            if recenter {
                let mut after = center;
                // center of obj after ops
                tf.map_point(&mut after);
                tf = tf.post_translate(before.x - after.x, before.y - after.y);
            }
        }

        if tf.is_identity() || !tf.is_valid() {
            return Ok(());
        }

        out.write_str(r#" transform="matrix("#)?;
        out.write_float(tf.sx)?;
        out.write_char(' ')?;
        out.write_float(tf.ky)?;
        out.write_char(' ')?;
        out.write_float(tf.kx)?;
        out.write_char(' ')?;
        out.write_float(tf.sy)?;
        out.write_char(' ')?;
        out.write_float(tf.tx)?;
        out.write_char(' ')?;
        out.write_float(tf.ty)?;
        out.write_str(r#")""#)
    }
}

/// Conversion trait for values that can be interpreted as an `(x, y)` float
/// pair.
pub trait IntoFloatPair {
    /// Converts the value into a pair of floats.
    fn into_float_pair(self) -> (f32, f32);
}

impl IntoFloatPair for f32 {
    fn into_float_pair(self) -> (f32, f32) {
        (self, self)
    }
}

impl IntoFloatPair for (f32, f32) {
    fn into_float_pair(self) -> (f32, f32) {
        (self.0, self.1)
    }
}

impl IntoFloatPair for [f32; 2] {
    fn into_float_pair(self) -> (f32, f32) {
        IntoFloatPair::into_float_pair((self[0], self[1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::str_sink;

    fn transform_sink(
        tf: Transform,
        pos: (f32, f32),
        translate: (f32, f32),
        size: (f32, f32),
    ) -> String {
        str_sink(|out| tf.write(out, pos, translate, size))
    }

    #[test]
    fn defaults_to_identity() {
        assert!(transform_sink(Transform::new(), (0.0, 0.0), (0.0, 0.0), (10.0, 10.0)).is_empty());
    }

    #[test]
    fn explicit_matrix() {
        assert_eq!(
            transform_sink(
                Transform::matrix(0.4, 0.3, 0.2, 0.5, 10.0, 15.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(0.4 0.3 0.2 0.5 10 15)""#
        );
    }

    //

    #[test]
    fn translates_xy() {
        assert_eq!(
            transform_sink(
                Transform::new().translate((1.0, 2.0)),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0),
            ),
            r#" transform="matrix(1 0 0 1 1 2)""#
        );
    }

    #[test]
    fn translates_x() {
        assert_eq!(
            transform_sink(
                Transform::new().translate_x(5.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0 0 1 5 0)""#
        );
    }

    #[test]
    fn translates_y() {
        assert_eq!(
            transform_sink(
                Transform::new().translate_y(5.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0 0 1 0 5)""#
        );
    }

    //

    #[test]
    fn scales_uniformly() {
        assert_eq!(
            transform_sink(
                Transform::new().scale(2.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(2.5 0 0 2.5 -7.5 -7.5)""#
        );
    }

    #[test]
    fn scales_xy() {
        assert_eq!(
            transform_sink(
                Transform::new().scale((2.5, 4.5)),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(2.5 0 0 4.5 -7.5 -17.5)""#
        );
    }

    #[test]
    fn scales_x() {
        assert_eq!(
            transform_sink(
                Transform::new().scale_x(2.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(2.5 0 0 1 -7.5 0)""#
        );
    }

    #[test]
    fn scales_y() {
        assert_eq!(
            transform_sink(
                Transform::new().scale_y(2.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0 0 2.5 0 -7.5)""#
        );
    }

    //

    #[test]
    fn rotates_about_center() {
        assert_eq!(
            transform_sink(
                Transform::new().rotate(45.0),
                (10.0, 10.0),
                (0.0, 0.0),
                (50.0, 50.0)
            ),
            r#" transform="matrix(0.7071 0.7071 -0.7071 0.7071 35 -14.4975)""#
        );
    }

    #[test]
    fn rotates_about_origin() {
        assert_eq!(
            transform_sink(
                Transform::new().rotate_origin(45.0),
                (10.0, 10.0),
                (0.0, 0.0),
                (50.0, 50.0)
            ),
            r#" transform="matrix(0.7071 0.7071 -0.7071 0.7071 0 0)""#
        );
    }

    #[test]
    fn rotates_at_point() {
        assert_eq!(
            transform_sink(
                Transform::new().rotate_at(45.0, 3.0, 5.0),
                (10.0, 10.0),
                (0.0, 0.0),
                (50.0, 50.0)
            ),
            r#" transform="matrix(0.7071 0.7071 -0.7071 0.7071 4.4142 -0.6569)""#
        );
    }

    //

    #[test]
    fn skews_uniformly() {
        assert_eq!(
            transform_sink(
                Transform::new().skew(0.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0.5 0.5 1 -2.5 -2.5)""#
        );
    }

    #[test]
    fn skews_xy() {
        assert_eq!(
            transform_sink(
                Transform::new().skew((0.5, 0.6)),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0.6 0.5 1 -2.5 -3)""#
        );
    }

    #[test]
    fn skews_x() {
        assert_eq!(
            transform_sink(
                Transform::new().skew_x(0.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0 0.5 1 -2.5 0)""#
        );
    }

    #[test]
    fn skews_y() {
        assert_eq!(
            transform_sink(
                Transform::new().skew_y(0.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (10.0, 10.0)
            ),
            r#" transform="matrix(1 0.5 0 1 0 -2.5)""#
        );
    }

    //

    #[test]
    fn into_float_pair() {
        assert_eq!(1.5_f32.into_float_pair(), (1.5, 1.5));
        assert_eq!((1.0, 2.0).into_float_pair(), (1.0, 2.0));
        assert_eq!([1.0, 2.0].into_float_pair(), (1.0, 2.0));
    }
}
