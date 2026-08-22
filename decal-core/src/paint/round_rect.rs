use crate::{
    paint::ScaledRadii,
    primitives::{
        Rect,
        Size,
    },
    utils::PathWriter,
};

/// Writes a path describing the filled area of a rectangle with optional
/// rounded corners.
///
/// # Arguments
/// - `out`: The output sink for the generated path commands.
/// - `w`: The width of the rectangle.
/// - `h`: The height of the rectangle.
/// - `r`: The [`ScaledRadii`] applied to the rectangle.
pub(crate) fn write_fill_path<T>(out: &mut T, w: f32, h: f32, r: ScaledRadii) -> std::fmt::Result
where
    T: std::fmt::Write,
{
    write_round_rect(out, 0.0, 0.0, w, h, r)
}

/// Writes a path describing the border area of a rectangle.
///
/// The path consists of an outer ring and an inner ring inset by the border
/// widths.
///
/// # Arguments
/// - `out`: The output sink for the generated path commands.
/// - `w`: The outer width of the rectangle.
/// - `h`: The outer height of the rectangle.
/// - `r`: The outer [`ScaledRadii`].
/// - `border`: The border widths.
pub(crate) fn write_border_path<T>(
    out: &mut T,
    w: f32,
    h: f32,
    r: ScaledRadii,
    border: Rect<f32>,
) -> std::fmt::Result
where
    T: std::fmt::Write,
{
    let (bt, br, bb, bl) = border.tuple();
    if bt + br + bb + bl == 0.0 {
        return Ok(());
    }

    write_round_rect(out, 0.0, 0.0, w, h, r)?; // outer ring
    write_round_rect(out, bl, bt, w - br, h - bb, inset_radii(r, border))?; // inner ring

    Ok(())
}

/// Writes a clipping path based on overflow rules and root scene bounds.
///
/// The resulting path restricts rendering to the specified axes while
/// respecting border insets and corner radii.
///
/// # Arguments
/// - `out`: The output sink for the generated path commands.
/// - `w`: The width of the element.
/// - `h`: The height of the element.
/// - `r`: The outer [`ScaledRadii`].
/// - `border`: The border widths applied to the element.
/// - `clip_x`: Whether clipping is applied along the horizontal axis.
/// - `clip_y`: Whether clipping is applied along the vertical axis.
/// - `scene_size`: The size of the root scene used when clipping extends beyond
///   the element.
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_clip_path<T>(
    out: &mut T,
    w: f32,
    h: f32,
    r: ScaledRadii,
    border: Rect<f32>,
    clip_x: bool,
    clip_y: bool,
    scene_size: Size<f32>,
) -> std::fmt::Result
where
    T: std::fmt::Write,
{
    let (bt, br, bb, bl) = border.tuple();
    let (scene_w, scene_h) = (scene_size.width(), scene_size.height());
    let ir = inset_radii(r, border);
    let (x1, y1) = (bl, bt);
    let (x2, y2) = (w - br, h - bb);

    match (clip_x, clip_y) {
        // no clipping
        (false, false) => Ok(()),
        // full clipping
        (true, true) => write_round_rect(out, x1, y1, x2, y2, ir),
        // clip x (entire height of element is visible, bounded by root height)
        (true, false) => write_round_rect(out, x1, 0.0, x2, scene_h, ir),
        // clip y (entire width of element is visible, bounded by root width)
        (false, true) => write_round_rect(out, 0.0, y1, scene_w, y2, ir),
    }
}

/// Computes [`ScaledRadii`] inset by the provided border widths.
///
/// # Note
/// Radii are clamped to zero to avoid negative values.
///
/// # Arguments
/// - `r`: The initial [`ScaledRadii`].
/// - `border`: The border widths.
///
/// # Returns
/// - The inset [`ScaledRadii`].
fn inset_radii(r: ScaledRadii, border: Rect<f32>) -> ScaledRadii {
    let (bt, br, bb, bl) = border.tuple();
    ScaledRadii {
        h_tl: (r.h_tl - bl).max(0.0),
        h_tr: (r.h_tr - br).max(0.0),
        h_br: (r.h_br - br).max(0.0),
        h_bl: (r.h_bl - bl).max(0.0),
        //
        v_tl: (r.v_tl - bt).max(0.0),
        v_tr: (r.v_tr - bt).max(0.0),
        v_br: (r.v_br - bb).max(0.0),
        v_bl: (r.v_bl - bb).max(0.0),
    }
}

/// Writes a rectangular path with optional rounded corners.
///
/// # Arguments
/// - `out`: The output sink for the generated path commands.
/// - `x1`: The left coordinate.
/// - `y1`: The top coordinate.
/// - `x2`: The right coordinate.
/// - `y2`: The bottom coordinate.
/// - `r`: The [`ScaledRadii`] value.
fn write_round_rect<T>(
    out: &mut T,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    r: ScaledRadii,
) -> std::fmt::Result
where
    T: std::fmt::Write,
{
    if r.h_tl + r.h_tr + r.h_br + r.h_bl == 0.0 {
        PathWriter::new(out)
            .move_to(x1, y1)?
            .horizontal_to(x2)?
            .vertical_to(y2)?
            .horizontal_to(x1)?
            .close()
    } else {
        PathWriter::new(out)
            .move_to(x1 + r.h_tl, y1)?
            .horizontal_to(x2 - r.h_tr)?
            .arc_to(r.h_tr, r.v_tr, x2, y1 + r.v_tr)?
            .vertical_to(y2 - r.v_br)?
            .arc_to(r.h_br, r.v_br, x2 - r.h_br, y2)?
            .horizontal_to(x1 + r.h_bl)?
            .arc_to(r.h_bl, r.v_bl, x1, y2 - r.v_bl)?
            .vertical_to(y1 + r.v_tl)?
            .arc_to(r.h_tl, r.v_tl, x1 + r.h_tl, y1)?
            .close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        paint::ScaledRadii,
        primitives::{
            Rect,
            Size,
        },
        test_utils::str_sink,
    };

    fn rounded_radii(x: f32) -> ScaledRadii {
        ScaledRadii {
            h_tl: x,
            h_tr: x,
            h_br: x,
            h_bl: x,
            v_tl: x,
            v_tr: x,
            v_br: x,
            v_bl: x,
        }
    }

    #[test]
    fn renders_fill_path_with_zero_radius() {
        assert_eq!(
            str_sink(|out| write_fill_path(out, 100.0, 50.0, ScaledRadii::default())),
            "M0 0 H100 V50 H0 Z"
        );
    }

    #[test]
    fn renders_fill_path_with_radius() {
        assert_eq!(
            str_sink(|out| write_fill_path(out, 100.0, 50.0, ScaledRadii::default())),
            "M0 0 H100 V50 H0 Z"
        );
    }

    //

    #[test]
    fn renders_border_path_with_zero_border() {
        assert!(
            str_sink(|out| write_border_path(
                out,
                100.0,
                50.0,
                rounded_radii(10.0),
                Rect::from_values(0.0, 0.0, 0.0, 0.0)
            ))
            .is_empty()
        );
    }

    #[test]
    fn renders_border_path() {
        assert_eq!(
            str_sink(|out| write_border_path(
                out,
                100.0,
                50.0,
                rounded_radii(10.0),
                Rect::from_values(5.0, 6.0, 7.0, 8.0),
            )),
            "M10 0 H90 A10 10 0 0 1 100 10 V40 A10 10 0 0 1 90 50 H10 A10 10 0 0 1 0 40 V10 A10 10 0 0 1 10 0 ZM10 5 H90 A4 5 0 0 1 94 10 V40 A4 3 0 0 1 90 43 H10 A2 3 0 0 1 8 40 V10 A2 5 0 0 1 10 5 Z"
        );
    }

    //

    #[test]
    fn renders_clip_path_with_no_clipping() {
        assert!(
            str_sink(|out| {
                write_clip_path(
                    out,
                    100.0,
                    50.0,
                    rounded_radii(10.0),
                    Rect::from_values(5.0, 6.0, 7.0, 8.0),
                    false,
                    false,
                    Size::from_values(200.0, 200.0),
                )
            })
            .is_empty()
        );
    }

    #[test]
    fn renders_clip_path_with_full_clipping() {
        assert_eq!(
            str_sink(|out| {
                write_clip_path(
                    out,
                    100.0,
                    50.0,
                    rounded_radii(10.0),
                    Rect::from_values(5.0, 6.0, 7.0, 8.0),
                    true,
                    true,
                    Size::from_values(200.0, 200.0),
                )
            }),
            "M10 5 H90 A4 5 0 0 1 94 10 V40 A4 3 0 0 1 90 43 H10 A2 3 0 0 1 8 40 V10 A2 5 0 0 1 10 5 Z"
        );
    }

    #[test]
    fn renders_clip_path_with_clip_x() {
        assert_eq!(
            str_sink(|out| {
                write_clip_path(
                    out,
                    100.0,
                    50.0,
                    rounded_radii(10.0),
                    Rect::from_values(5.0, 6.0, 7.0, 8.0),
                    true,
                    false,
                    Size::from_values(200.0, 200.0),
                )
            }),
            "M10 0 H90 A4 5 0 0 1 94 5 V197 A4 3 0 0 1 90 200 H10 A2 3 0 0 1 8 197 V5 A2 5 0 0 1 10 0 Z"
        );
    }

    #[test]
    fn renders_clip_path_with_clip_y() {
        assert_eq!(
            str_sink(|out| {
                write_clip_path(
                    out,
                    100.0,
                    50.0,
                    rounded_radii(10.0),
                    Rect::from_values(5.0, 6.0, 7.0, 8.0),
                    false,
                    true,
                    Size::from_values(200.0, 200.0),
                )
            }),
            "M2 5 H196 A4 5 0 0 1 200 10 V40 A4 3 0 0 1 196 43 H2 A2 3 0 0 1 0 40 V10 A2 5 0 0 1 2 5 Z"
        );
    }
}
