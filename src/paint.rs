//! Painting Material shapes with Rune: [`rune::Shape`]s and [`rune::El`]s.
//!
//! Shapes are normalized to the unit square, so a filled shape uses a `1 × 1` view box and
//! stretches to whatever square the canvas lays out at, like Compose's `RoundedPolygon.toShape`.
//! Outlines are stroked in points, so a hairline stays a hairline at any size.

use rune::{Color, El, LineCap, Shape, canvas};

use crate::{MaterialShape, Morph, RoundedPolygon};

/// The flattening tolerance used for painted shapes: curves stay within this of the true outline
/// in unit-square coordinates, about half a pixel on a 320 pt canvas.
pub const PAINT_TOLERANCE: f32 = 0.0015;

/// How a Material shape is painted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintStyle {
    /// The shape's inside, filled with one colour.
    #[default]
    Filled,
    /// Its outline, stroked.
    Outlined,
    /// A translucent fill under a solid outline.
    Soft,
}

/// `polygon` as a Rune [`Shape`], ready for [`rune::canvas`].
pub fn paint(polygon: &RoundedPolygon, style: PaintStyle, color: Color) -> Shape {
    let outline = polygon.outline(PAINT_TOLERANCE);
    let shape = Shape::new().view_box(1.0, 1.0);
    match style {
        PaintStyle::Filled => shape.fill(outline, color),
        PaintStyle::Outlined => shape.stroke_with(outline, true, 1.5, LineCap::Round, color),
        PaintStyle::Soft => shape.fill(outline.clone(), color.alpha(0.22)).stroke_with(
            outline,
            true,
            1.5,
            LineCap::Round,
            color,
        ),
    }
}

/// The morph at `progress` as a Rune [`Shape`]: the same paint as [`paint`], for a surface that
/// changes shape every frame.
pub fn morph_paint(morph: &Morph, progress: f32, style: PaintStyle, color: Color) -> Shape {
    paint(&morph.polygon(progress), style, color)
}

/// `el` with the morph at `progress` painted beneath its content (as [`rune::El::shape`] does).
/// The shape stretches to the element's rect. Rune doesn't clip content to arbitrary paths, so the
/// content is not clipped to the silhouette and the element's hit area stays rectangular; keep
/// content inset.
pub fn shaped(el: El, morph: &Morph, progress: f32, style: PaintStyle, color: Color) -> El {
    el.shape(morph_paint(morph, progress, style, color))
}

/// A square canvas `side` points wide showing `shape`.
pub fn shape_canvas(shape: MaterialShape, side: f32, style: PaintStyle, color: Color) -> El {
    canvas(paint(&shape.polygon(), style, color)).size(side, side)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_paints_in_every_style() {
        for shape in MaterialShape::ALL {
            for style in [PaintStyle::Filled, PaintStyle::Outlined, PaintStyle::Soft] {
                let painted = paint(&shape.polygon(), style, Color::WHITE);
                assert!(!painted.is_empty(), "{} {style:?}", shape.name());
            }
        }
    }

    #[test]
    fn a_morph_paints_at_its_endpoints_and_in_between() {
        let morph = Morph::new(
            MaterialShape::Square.polygon(),
            MaterialShape::Cookie6Sided.polygon(),
        );
        for progress in [0.0f32, 0.5, 1.0] {
            assert!(!morph_paint(&morph, progress, PaintStyle::Filled, Color::WHITE).is_empty());
        }
    }
}
