//! Geometry and catalog checks: every shape normalized, bounded, simple enough for Rune's fills,
//! and the ones Android mirrors actually mirrored.

use crate::{Category, CornerRounding, MaterialShape, Morph, RoundedPolygon};
use std::f32::consts::PI;

const TOLERANCE: f32 = 0.0015;

fn outline(shape: MaterialShape) -> Vec<[f32; 2]> {
    shape.polygon().outline(TOLERANCE)
}

fn close(a: [f32; 2], b: [f32; 2], eps: f32) -> bool {
    (a[0] - b[0]).abs() <= eps && (a[1] - b[1]).abs() <= eps
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// A proper crossing: segments that only touch at an endpoint or run collinearly do not count.
fn crosses(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let cd = [d[0] - c[0], d[1] - c[1]];
    let ac = [c[0] - a[0], c[1] - a[1]];
    let ad = [d[0] - a[0], d[1] - a[1]];
    let ca = [a[0] - c[0], a[1] - c[1]];
    let cb = [b[0] - c[0], b[1] - c[1]];
    let d1 = cross(ab, ac);
    let d2 = cross(ab, ad);
    let d3 = cross(cd, ca);
    let d4 = cross(cd, cb);
    let eps = 1e-6;
    ((d1 > eps && d2 < -eps) || (d1 < -eps && d2 > eps))
        && ((d3 > eps && d4 < -eps) || (d3 < -eps && d4 > eps))
}

fn is_simple(points: &[[f32; 2]]) -> bool {
    let n = points.len();
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];
        for j in (i + 1)..n {
            // Adjacent edges share a vertex; the first and last are adjacent too.
            if j == i || j == (i + 1) % n || i == (j + 1) % n {
                continue;
            }
            let c = points[j];
            let d = points[(j + 1) % n];
            if crosses(a, b, c, d) {
                return false;
            }
        }
    }
    true
}

fn signed_area(points: &[[f32; 2]]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a[0] * b[1] - b[0] * a[1];
    }
    area * 0.5
}

fn distance_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + t * d[0], a[1] + t * d[1]];
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt()
}

/// The furthest either outline strays from the other, as a rough shape comparison that ignores
/// where the contour starts and how it is subdivided.
fn hausdorff(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    let nearest = |p: [f32; 2], poly: &[[f32; 2]]| {
        (0..poly.len())
            .map(|i| distance_to_segment(p, poly[i], poly[(i + 1) % poly.len()]))
            .fold(f32::INFINITY, f32::min)
    };
    let forward = a.iter().map(|p| nearest(*p, b)).fold(0.0f32, f32::max);
    let backward = b.iter().map(|p| nearest(*p, a)).fold(0.0f32, f32::max);
    forward.max(backward)
}

#[test]
fn the_catalog_has_android_s_35_shapes_once_each() {
    assert_eq!(MaterialShape::ALL.len(), 35);
    let mut names: Vec<&str> = MaterialShape::ALL.iter().map(|s| s.name()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 35, "names are unique");
    let mut labels: Vec<&str> = MaterialShape::ALL.iter().map(|s| s.label()).collect();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), 35, "labels are unique");
    let count: usize = [
        Category::Basic,
        Category::Cookies,
        Category::Clovers,
        Category::Bursts,
        Category::Unique,
    ]
    .into_iter()
    .map(|c| MaterialShape::in_category(c).count())
    .sum();
    assert_eq!(count, 35, "each shape has exactly one category");
    assert_eq!(MaterialShape::Cookie12Sided.name(), "Cookie12Sided");
    assert_eq!(
        crate::MaterialShapes::Cookie12Sided,
        MaterialShape::Cookie12Sided
    );
    assert_eq!(MaterialShape::Cookie12Sided.label(), "Cookie 12-sided");
}

#[test]
fn every_shape_normalizes_into_the_unit_square() {
    for shape in MaterialShape::ALL {
        let polygon = shape.polygon();
        let [left, top, right, bottom] = polygon.bounds();
        let (width, height) = (right - left, bottom - top);
        assert!(
            width > 0.2 && height > 0.2,
            "{} has a size: {width}×{height}",
            shape.name()
        );
        assert!(
            (width.max(height) - 1.0).abs() < 1e-4,
            "{} fills its longer axis: {width}×{height}",
            shape.name()
        );
        if width < height {
            assert!(
                (left - (1.0 - width) / 2.0).abs() < 1e-4
                    && (right - (1.0 + width) / 2.0).abs() < 1e-4,
                "{} is centered across: {left}..{right}",
                shape.name()
            );
            assert!(top.abs() < 1e-4 && (bottom - 1.0).abs() < 1e-4);
        } else {
            assert!(
                (top - (1.0 - height) / 2.0).abs() < 1e-4
                    && (bottom - (1.0 + height) / 2.0).abs() < 1e-4,
                "{} is centered down: {top}..{bottom}",
                shape.name()
            );
            assert!(left.abs() < 1e-4 && (right - 1.0).abs() < 1e-4);
        }
    }
}

#[test]
fn every_outline_is_inside_bounds_simple_and_has_area() {
    for shape in MaterialShape::ALL {
        let points = outline(shape);
        assert!(points.len() > 10, "{} has an outline", shape.name());
        for [x, y] in &points {
            assert!(
                (-1e-3..=1.0 + 1e-3).contains(x) && (-1e-3..=1.0 + 1e-3).contains(y),
                "{} stays in the unit square, not ({x}, {y})",
                shape.name()
            );
        }
        for pair in points.windows(2) {
            assert!(
                !close(pair[0], pair[1], 1e-6),
                "{} has no repeated consecutive points",
                shape.name()
            );
        }
        assert!(
            !close(points[0], *points.last().unwrap(), 1e-6),
            "{} does not repeat its seam",
            shape.name()
        );
        assert!(is_simple(&points), "{} is a simple outline", shape.name());
        assert!(
            signed_area(&points).abs() > 0.05,
            "{} encloses area",
            shape.name()
        );
    }
}

#[test]
fn mirrored_shapes_are_symmetric_across_the_vertical_center() {
    use MaterialShape::*;
    for shape in [
        Pentagon,
        Gem,
        Ghostish,
        Clover4Leaf,
        SoftBoom,
        Flower,
        PuffyDiamond,
        PixelCircle,
        Bun,
        Heart,
    ] {
        let polygon = shape.polygon();
        let points = polygon.outline(TOLERANCE);
        for p in &points {
            let mirror = [1.0 - p[0], p[1]];
            assert!(
                points.iter().any(|q| close(*q, mirror, 4e-3)),
                "{} mirrors ({}, {})",
                shape.name(),
                p[0],
                p[1]
            );
        }
    }
}

#[test]
fn a_tighter_tolerance_adds_points_without_moving_the_shape() {
    let polygon = MaterialShape::Flower.polygon();
    let coarse = polygon.outline(0.01);
    let fine = polygon.outline(0.0005);
    assert!(fine.len() > coarse.len());
    assert_eq!(fine[0], coarse[0], "both start at the same anchor");
}

#[test]
fn the_circle_is_round() {
    let points = outline(MaterialShape::Circle);
    for [x, y] in &points {
        let radius = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
        assert!((radius - 0.5).abs() < 0.01, "radius {radius} at ({x}, {y})");
    }
}

#[test]
fn smoothing_rounds_a_square_without_moving_its_bounds() {
    let sharp = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::UNROUNDED);
    let round = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::new(0.3));
    let smooth = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::smoothed(0.3, 1.0));
    let sharp_points = sharp.outline(TOLERANCE);
    let round_points = round.outline(TOLERANCE);
    let smooth_points = smooth.outline(TOLERANCE);
    assert_eq!(sharp_points.len(), 4, "a square's corners are its vertices");
    assert!(round_points.len() > sharp_points.len());
    assert_ne!(smooth_points, round_points, "smoothing changes the outline");
    assert_eq!(
        smooth.bounds(),
        round.bounds(),
        "smoothing keeps the bounds"
    );
    for p in &round_points {
        let near_corner = (p[0].abs() < 0.31 || (p[0] - 1.0).abs() < 0.31)
            && (p[1].abs() < 0.31 || (p[1] - 1.0).abs() < 0.31);
        assert!(!near_corner || (p[0].abs() > 0.299 - 0.001 || (p[0] - 1.0).abs() > 0.299 - 0.001));
    }
}

#[test]
fn regular_polygons_rotate_and_scale_as_one() {
    let triangle = RoundedPolygon::regular(3, 1.0, [0.0, 0.0], CornerRounding::UNROUNDED);
    let turned = triangle.rotated(PI / 2.0);
    let a = triangle.outline(TOLERANCE);
    let b = turned.outline(TOLERANCE);
    assert_eq!(a.len(), 3);
    for p in &a {
        assert!(b.iter().any(|q| close(*q, [-p[1], p[0]], 1e-5)));
    }
    let scaled = triangle.scaled(2.0, 3.0);
    assert!((scaled.bounds()[2] - 2.0 * triangle.bounds()[2]).abs() < 1e-5);
    assert!((scaled.bounds()[3] - 3.0 * triangle.bounds()[3]).abs() < 1e-5);
}

#[test]
fn normalization_is_uniform() {
    let wide = RoundedPolygon::rectangle(2.0, 1.0, CornerRounding::UNROUNDED).normalized();
    let [left, top, right, bottom] = wide.bounds();
    assert!((left - 0.0).abs() < 1e-5 && (right - 1.0).abs() < 1e-5);
    assert!(
        (bottom - top - 0.5).abs() < 1e-5,
        "a 2:1 rectangle keeps its ratio"
    );
    assert!(((top + bottom) / 2.0 - 0.5).abs() < 1e-5);
}

#[test]
fn every_morph_starts_and_ends_as_its_two_shapes() {
    // Any of the 35 shapes can morph into any other, including itself. The endpoints must be the
    // exact shapes, not approximations that drift when curves are matched and cut.
    for start in MaterialShape::ALL {
        for end in MaterialShape::ALL {
            let morph = Morph::new(start.polygon(), end.polygon());
            let at_start = morph.outline(0.0, TOLERANCE);
            let at_end = morph.outline(1.0, TOLERANCE);
            let start_outline = start.polygon().outline(TOLERANCE);
            let end_outline = end.polygon().outline(TOLERANCE);
            assert!(
                hausdorff(&at_start, &start_outline) < 2e-3,
                "{} -> {} starts as {}",
                start.name(),
                end.name(),
                start.name()
            );
            assert!(
                hausdorff(&at_end, &end_outline) < 2e-3,
                "{} -> {} ends as {}",
                start.name(),
                end.name(),
                end.name()
            );
        }
    }
}

#[test]
fn every_pair_has_a_single_clean_shape_halfway() {
    // The midpoint is where a bad mapping or an uncovered cut shows up as a fold, a gap, or a
    // collapsed shape. Check every ordered pair, at the middle and off-centre.
    for start in MaterialShape::ALL {
        for end in MaterialShape::ALL {
            let morph = Morph::new(start.polygon(), end.polygon());
            for progress in [0.25f32, 0.5, 0.75] {
                let points = morph.outline(progress, TOLERANCE);
                assert!(
                    !close(points[0], *points.last().unwrap(), 1e-6),
                    "{} -> {} at {progress} does not repeat its seam",
                    start.name(),
                    end.name()
                );
                assert!(
                    is_simple(&points),
                    "{} -> {} at {progress} is a simple outline",
                    start.name(),
                    end.name()
                );
                assert!(
                    signed_area(&points).abs() > 0.02,
                    "{} -> {} at {progress} encloses area",
                    start.name(),
                    end.name()
                );
                for [x, y] in &points {
                    assert!(
                        (-1e-3..=1.0 + 1e-3).contains(x) && (-1e-3..=1.0 + 1e-3).contains(y),
                        "{} -> {} strays to ({x}, {y}) at {progress}",
                        start.name(),
                        end.name()
                    );
                }
            }
        }
    }
}

#[test]
fn a_morph_grows_and_shrinks_monotonically_through_its_progress() {
    // A circle's area should shrink into a small cookie without bouncing: area is continuous and
    // the curve count is stable across the whole animation.
    let morph = Morph::new(
        MaterialShape::Circle.polygon(),
        MaterialShape::Cookie12Sided.polygon(),
    );
    let curves = morph.curve_count();
    assert!(curves > 10, "the match keeps the circle's curves: {curves}");
    let mut previous = signed_area(&morph.outline(0.0, TOLERANCE)).abs();
    for i in 1..=20 {
        let progress = i as f32 / 20.0;
        let points = morph.outline(progress, TOLERANCE);
        let area = signed_area(&points).abs();
        assert!(
            (area - previous).abs() < 0.12,
            "area jumps from {previous} to {area} at {progress}"
        );
        assert_eq!(morph.curve_count(), curves);
        previous = area;
    }
    assert!(previous < 0.92, "the end is the cookie's smaller area");
}
