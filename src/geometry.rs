// Copyright 2022 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Ported from androidx.graphics.shapes.RoundedPolygon, modified here; see README.md.

//! Rounded polygons: the geometry behind Material 3 Expressive's shapes.
//!
//! A [`RoundedPolygon`] is a closed contour of cubic Bézier curves: straight edges between the
//! vertices, with each vertex optionally rounded by a circular arc and smoothed into the edges.
//! This is a direct port of `androidx.graphics.shapes.RoundedPolygon`, the model Android uses for
//! `androidx.compose.material3.MaterialShapes`, so the same vertices, radii and smoothing values
//! draw the same outlines here.
//!
//! Coordinates are unitless. Material's shapes are built around the origin (or `(0.5, 0.5)`) and
//! then [`normalized`](RoundedPolygon::normalized) into the unit square, ready to be painted at
//! any size. See [`crate::MaterialShape`] for the 35 named shapes.
//!
//! ```
//! use rune_material_shapes::{CornerRounding, RoundedPolygon};
//!
//! // A square with 30% corner rounding, like MaterialShapes.Square.
//! let square = RoundedPolygon::rectangle(1.0, 1.0, CornerRounding::new(0.3)).normalized();
//! let outline = square.outline(0.002);
//! assert!(outline.len() > 8);
//! for [x, y] in outline {
//!     assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));
//! }
//! ```

use std::f32::consts::PI;

use crate::math::{
    add, average, cross, direction, dot, is_convex, length, lerp, mul, rotate90, sub,
};

/// Points closer than this are the same point, as in `androidx.graphics.shapes`.
pub const DISTANCE_EPSILON: f32 = 1e-4;

/// How one vertex is rounded.
///
/// `radius` is the radius of the circular arc in the polygon's own coordinates, and `smoothing`
/// extends that arc into the adjacent edges with two flanking curves: `0.0` is a plain circular
/// corner, and `1.0` removes the arc so the flanks meet in the middle. Material's shapes use both:
/// [`CornerRounding::new`] for cookies and clovers, and [`CornerRounding::smoothed`] for shapes
/// such as the slanted square and the gem.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRounding {
    pub radius: f32,
    pub smoothing: f32,
}

impl CornerRounding {
    /// No rounding: a sharp vertex.
    pub const UNROUNDED: CornerRounding = CornerRounding {
        radius: 0.0,
        smoothing: 0.0,
    };

    /// A circular corner of `radius`.
    pub const fn new(radius: f32) -> CornerRounding {
        CornerRounding {
            radius,
            smoothing: 0.0,
        }
    }

    /// A circular corner of `radius` whose curves flow into the edges by `smoothing` (0 to 1).
    pub const fn smoothed(radius: f32, smoothing: f32) -> CornerRounding {
        CornerRounding { radius, smoothing }
    }
}

/// What a [`Feature`] is: a straight edge, or a corner that turns outward or inward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureKind {
    Edge,
    ConvexCorner,
    ConcaveCorner,
}

/// A contiguous run of cubics with a role, as in Android's `RoundedPolygon.features`: the curves
/// around one vertex, or the edge to the next one. Morphing matches these instead of raw cubics,
/// so a rounded rectangle's corner still maps to one corner.
#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    pub kind: FeatureKind,
    pub cubics: Vec<Cubic>,
}

impl Feature {
    pub fn is_corner(&self) -> bool {
        self.kind != FeatureKind::Edge
    }

    /// The midpoint between this feature's first and last anchor, as Android's
    /// `featureRepresentativePoint`.
    pub(crate) fn representative_point(&self) -> [f32; 2] {
        let first = self.cubics.first().expect("a feature has cubics").anchor0;
        let last = self.cubics.last().expect("a feature has cubics").anchor1;
        [(first[0] + last[0]) / 2.0, (first[1] + last[1]) / 2.0]
    }
}

/// A single cubic Bézier segment from `anchor0` to `anchor1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cubic {
    pub anchor0: [f32; 2],
    pub control0: [f32; 2],
    pub control1: [f32; 2],
    pub anchor1: [f32; 2],
}

impl Cubic {
    pub const fn new(
        anchor0: [f32; 2],
        control0: [f32; 2],
        control1: [f32; 2],
        anchor1: [f32; 2],
    ) -> Cubic {
        Cubic {
            anchor0,
            control0,
            control1,
            anchor1,
        }
    }

    /// The straight line from `a` to `b`, as a cubic with its controls a third and two thirds in.
    pub fn straight_line(a: [f32; 2], b: [f32; 2]) -> Cubic {
        Cubic::new(a, lerp(a, b, 1.0 / 3.0), lerp(a, b, 2.0 / 3.0), b)
    }

    /// The smaller circular arc around `center` from `a` to `b`, approximated with one cubic.
    fn circular_arc(center: [f32; 2], a: [f32; 2], b: [f32; 2]) -> Cubic {
        let (Some(p0d), Some(p1d)) = (direction(sub(a, center)), direction(sub(b, center))) else {
            return Cubic::straight_line(a, b);
        };
        let rotated_p0 = rotate90(p0d);
        let rotated_p1 = rotate90(p1d);
        let clockwise = dot(rotated_p0, sub(b, center)) >= 0.0;
        let cosa = dot(p0d, p1d);
        if cosa > 0.999 {
            return Cubic::straight_line(a, b);
        }
        let k = length(sub(a, center)) * 4.0 / 3.0
            * ((2.0 * (1.0 - cosa)).sqrt() - (1.0 - cosa * cosa).sqrt())
            / (1.0 - cosa)
            * if clockwise { 1.0 } else { -1.0 };
        Cubic::new(a, add(a, mul(rotated_p0, k)), sub(b, mul(rotated_p1, k)), b)
    }

    /// This curve, from its end back to its start.
    pub fn reversed(self) -> Cubic {
        Cubic::new(self.anchor1, self.control1, self.control0, self.anchor0)
    }

    /// The point at `t` along the curve.
    pub fn point_at(self, t: f32) -> [f32; 2] {
        let u = 1.0 - t;
        let (b0, b1, b2, b3) = (u * u * u, 3.0 * t * u * u, 3.0 * t * t * u, t * t * t);
        [
            self.anchor0[0] * b0
                + self.control0[0] * b1
                + self.control1[0] * b2
                + self.anchor1[0] * b3,
            self.anchor0[1] * b0
                + self.control0[1] * b1
                + self.control1[1] * b2
                + self.anchor1[1] * b3,
        ]
    }

    /// Whether both anchors are within [`DISTANCE_EPSILON`].
    pub fn is_zero_length(self) -> bool {
        (self.anchor0[0] - self.anchor1[0]).abs() < DISTANCE_EPSILON
            && (self.anchor0[1] - self.anchor1[1]).abs() < DISTANCE_EPSILON
    }

    /// Splits the curve at `t` into two curves that together trace the same path.
    pub fn split(self, t: f32) -> (Cubic, Cubic) {
        let u = 1.0 - t;
        let p = self.point_at(t);
        (
            Cubic::new(
                self.anchor0,
                lerp(self.anchor0, self.control0, t),
                add(
                    add(mul(self.anchor0, u * u), mul(self.control0, 2.0 * u * t)),
                    mul(self.control1, t * t),
                ),
                p,
            ),
            Cubic::new(
                p,
                add(
                    add(mul(self.control0, u * u), mul(self.control1, 2.0 * u * t)),
                    mul(self.anchor1, t * t),
                ),
                lerp(self.control1, self.anchor1, t),
                self.anchor1,
            ),
        )
    }

    /// How far the controls stray from the chord, the measure used to stop flattening.
    fn flatness(self) -> f32 {
        let chord = sub(self.anchor1, self.anchor0);
        let len = length(chord);
        if len < DISTANCE_EPSILON {
            return length(sub(self.control0, self.anchor0))
                .max(length(sub(self.control1, self.anchor0)));
        }
        (cross(chord, sub(self.control0, self.anchor0)).abs() / len)
            .max(cross(chord, sub(self.control1, self.anchor0)).abs() / len)
    }

    /// Appends the curve's end point (and any subdivision points) to `out`, within `tolerance`.
    fn flatten_into(self, tolerance: f32, depth: u32, out: &mut Vec<[f32; 2]>) {
        if depth >= 10 || self.flatness() <= tolerance {
            out.push(self.anchor1);
            return;
        }
        let (a, b) = self.split(0.5);
        a.flatten_into(tolerance, depth + 1, out);
        b.flatten_into(tolerance, depth + 1, out);
    }
}

/// A closed contour of cubics, with the center used by transforms.
#[derive(Clone, Debug, PartialEq)]
pub struct RoundedPolygon {
    features: Vec<Feature>,
    cubics: Vec<Cubic>,
    center: [f32; 2],
}

impl RoundedPolygon {
    /// A polygon through `vertices` with every corner rounded by `rounding`.
    pub fn from_vertices(vertices: &[[f32; 2]], rounding: CornerRounding) -> RoundedPolygon {
        RoundedPolygon::from_vertices_with(vertices, &vec![rounding; vertices.len()], None)
    }

    /// A polygon through `vertices` with one [`CornerRounding`] per vertex.
    ///
    /// `center` is the point transforms rotate and scale around; without one, the average of the
    /// vertices is used, as in Android.
    pub fn from_vertices_with(
        vertices: &[[f32; 2]],
        per_vertex: &[CornerRounding],
        center: Option<[f32; 2]>,
    ) -> RoundedPolygon {
        assert!(
            vertices.len() >= 3,
            "a polygon needs at least three vertices"
        );
        assert_eq!(
            per_vertex.len(),
            vertices.len(),
            "one CornerRounding per vertex"
        );
        let n = vertices.len();
        let corners: Vec<RoundedCorner> = (0..n)
            .map(|i| {
                RoundedCorner::new(
                    vertices[(i + n - 1) % n],
                    vertices[i],
                    vertices[(i + 1) % n],
                    per_vertex[i],
                )
            })
            .collect();

        // How much of each side each corner may cut: rounding first, then smoothing.
        let cut_adjusts: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let a = &corners[i];
                let b = &corners[(i + 1) % n];
                let expected_round_cut = a.expected_round_cut + b.expected_round_cut;
                let expected_cut = a.expected_cut() + b.expected_cut();
                let side = length(sub(vertices[i], vertices[(i + 1) % n]));
                if expected_round_cut > side {
                    (side / expected_round_cut, 0.0)
                } else if expected_cut > side {
                    (
                        1.0,
                        (side - expected_round_cut) / (expected_cut - expected_round_cut),
                    )
                } else {
                    (1.0, 1.0)
                }
            })
            .collect();

        let mut corner_cubics: Vec<Vec<Cubic>> = Vec::with_capacity(n);
        let mut corner_kinds: Vec<FeatureKind> = Vec::with_capacity(n);
        for i in 0..n {
            let corner = &corners[i];
            let mut allowed = [0.0f32; 2];
            for (delta, slot) in allowed.iter_mut().enumerate() {
                let (round_ratio, cut_ratio) = cut_adjusts[(i + n - 1 + delta) % n];
                *slot = corner.expected_round_cut * round_ratio
                    + (corner.expected_cut() - corner.expected_round_cut) * cut_ratio;
            }
            // A zero-length corner is a sharp vertex, but it still keeps its place as a feature
            // so morphing can match it.
            corner_cubics.push(corner.cubics(allowed[0], allowed[1]));
            let prev = vertices[(i + n - 1) % n];
            let next = vertices[(i + 1) % n];
            corner_kinds.push(if is_convex(prev, vertices[i], next) {
                FeatureKind::ConvexCorner
            } else {
                FeatureKind::ConcaveCorner
            });
        }

        let starts: Vec<[f32; 2]> = corner_cubics
            .iter()
            .map(|c| {
                c.first()
                    .expect("a rounded corner has at least one cubic")
                    .anchor0
            })
            .collect();
        let ends: Vec<[f32; 2]> = corner_cubics
            .iter()
            .map(|c| {
                c.last()
                    .expect("a rounded corner has at least one cubic")
                    .anchor1
            })
            .collect();
        let mut features = Vec::with_capacity(n * 2);
        for i in 0..n {
            let end = ends[i];
            let next_start = starts[(i + 1) % n];
            features.push(Feature {
                kind: corner_kinds[i],
                cubics: std::mem::take(&mut corner_cubics[i]),
            });
            features.push(Feature {
                kind: FeatureKind::Edge,
                cubics: vec![Cubic::straight_line(end, next_start)],
            });
        }
        RoundedPolygon::from_features(features, center.unwrap_or_else(|| average(vertices)))
    }

    /// The polygon's features, in contour order, as Android's `RoundedPolygon.features`.
    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    fn from_features(features: Vec<Feature>, center: [f32; 2]) -> RoundedPolygon {
        // AndroidX starts the contour halfway through the first corner and drops zero-length
        // curves. Both matter when painting: starting exactly on a corner seam, or emitting a
        // run of zero-length curves, produces visible artifacts.
        let (first_start, first_end) = if features.first().is_some_and(|f| f.cubics.len() == 3) {
            let middle = features[0].cubics[1];
            let (start, end) = middle.split(0.5);
            (
                Some(vec![features[0].cubics[0], start]),
                Some(vec![end, features[0].cubics[2]]),
            )
        } else {
            (None, None)
        };

        let mut cubics: Vec<Cubic> = Vec::new();
        let mut first: Option<Cubic> = None;
        let mut last: Option<Cubic> = None;
        for i in 0..=features.len() {
            let feature_cubics: &[Cubic] = if i == 0 && first_end.is_some() {
                first_end.as_deref().unwrap_or_default()
            } else if i == features.len() {
                match first_start.as_deref() {
                    Some(start) => start,
                    None => break,
                }
            } else {
                &features[i].cubics
            };
            for &cubic in feature_cubics {
                if !cubic.is_zero_length() {
                    if let Some(previous) = last {
                        cubics.push(previous);
                    }
                    last = Some(cubic);
                    first.get_or_insert(cubic);
                } else if let Some(previous) = last.as_mut() {
                    // Keep the contour continuous across dropped curves, as AndroidX does.
                    previous.anchor1 = cubic.anchor1;
                }
            }
        }
        if let (Some(last), Some(first)) = (last, first) {
            cubics.push(Cubic::new(
                last.anchor0,
                last.control0,
                last.control1,
                first.anchor0,
            ));
        } else {
            cubics.push(Cubic::new(center, center, center, center));
        }
        RoundedPolygon {
            features,
            cubics,
            center,
        }
    }

    /// A polygon from raw cubics, one edge feature each. Morph results use this for painting.
    pub(crate) fn from_cubics(cubics: Vec<Cubic>, center: [f32; 2]) -> RoundedPolygon {
        let features = cubics
            .iter()
            .map(|c| Feature {
                kind: FeatureKind::Edge,
                cubics: vec![*c],
            })
            .collect();
        RoundedPolygon {
            features,
            cubics,
            center,
        }
    }

    /// A regular `num_vertices`-gon around `center`.
    pub fn regular(
        num_vertices: usize,
        radius: f32,
        center: [f32; 2],
        rounding: CornerRounding,
    ) -> RoundedPolygon {
        RoundedPolygon::regular_with(num_vertices, radius, center, &vec![rounding; num_vertices])
    }

    /// A regular polygon with one [`CornerRounding`] per vertex, in counter-clockwise order from
    /// the positive x axis.
    pub fn regular_with(
        num_vertices: usize,
        radius: f32,
        center: [f32; 2],
        per_vertex: &[CornerRounding],
    ) -> RoundedPolygon {
        assert!(num_vertices >= 3, "a polygon needs at least three vertices");
        let vertices: Vec<[f32; 2]> = (0..num_vertices)
            .map(|i| {
                let angle = 2.0 * PI * i as f32 / num_vertices as f32;
                [
                    center[0] + angle.cos() * radius,
                    center[1] + angle.sin() * radius,
                ]
            })
            .collect();
        RoundedPolygon::from_vertices_with(&vertices, per_vertex, Some(center))
    }

    /// A circle approximated by a `num_vertices`-gon whose corners are rounded to `radius`.
    ///
    /// Android's `MaterialShapes.Circle` uses ten vertices; `circle(8, 1.0)` matches
    /// `RoundedPolygon.circle()`.
    pub fn circle(num_vertices: usize, radius: f32) -> RoundedPolygon {
        assert!(num_vertices >= 3, "a circle needs at least three vertices");
        let theta = PI / num_vertices as f32;
        let polygon_radius = radius / theta.cos();
        RoundedPolygon::regular(
            num_vertices,
            polygon_radius,
            [0.0, 0.0],
            CornerRounding::new(radius),
        )
    }

    /// A rectangle centered on the origin, every corner rounded by `rounding`.
    pub fn rectangle(width: f32, height: f32, rounding: CornerRounding) -> RoundedPolygon {
        RoundedPolygon::rectangle_with(width, height, &[rounding; 4])
    }

    /// A rectangle centered on the origin with one [`CornerRounding`] per corner, clockwise from
    /// the bottom-right: bottom-right, bottom-left, top-left, top-right.
    pub fn rectangle_with(
        width: f32,
        height: f32,
        per_vertex: &[CornerRounding],
    ) -> RoundedPolygon {
        assert_eq!(per_vertex.len(), 4, "a rectangle has four corners");
        let (right, left) = (width / 2.0, -width / 2.0);
        let (bottom, top) = (height / 2.0, -height / 2.0);
        let vertices = [[right, bottom], [left, bottom], [left, top], [right, top]];
        RoundedPolygon::from_vertices_with(&vertices, per_vertex, Some([0.0, 0.0]))
    }

    /// A star with `num_vertices_per_radius` points, alternating between `radius` and `inner_radius`.
    pub fn star(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
    ) -> RoundedPolygon {
        assert!(
            num_vertices_per_radius >= 3,
            "a star needs at least three points"
        );
        assert!(
            radius > 0.0 && inner_radius > 0.0 && inner_radius < radius,
            "a star's radii must be positive, with a smaller inner radius"
        );
        let n = num_vertices_per_radius;
        let mut vertices = Vec::with_capacity(n * 2);
        for i in 0..n {
            let outer = 2.0 * PI * i as f32 / n as f32;
            let inner = PI * (2 * i + 1) as f32 / n as f32;
            vertices.push([outer.cos() * radius, outer.sin() * radius]);
            vertices.push([inner.cos() * inner_radius, inner.sin() * inner_radius]);
        }
        RoundedPolygon::from_vertices_with(
            &vertices,
            &vec![rounding; vertices.len()],
            Some([0.0, 0.0]),
        )
    }

    pub fn cubics(&self) -> &[Cubic] {
        &self.cubics
    }

    /// The point transforms use, and the point [`normalized`](Self::normalized) centers around.
    pub fn center(&self) -> [f32; 2] {
        self.center
    }

    /// The axis-aligned bounds of every anchor and control point, as `[left, top, right, bottom]`.
    ///
    /// Like Android's `calculateBounds(approximate = true)`, this is the control-polygon box: a
    /// little larger than the drawn curve, and what normalization uses.
    pub fn bounds(&self) -> [f32; 4] {
        let mut min = [f32::MAX; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for cubic in &self.cubics {
            for p in [cubic.anchor0, cubic.control0, cubic.control1, cubic.anchor1] {
                min[0] = min[0].min(p[0]);
                min[1] = min[1].min(p[1]);
                max[0] = max[0].max(p[0]);
                max[1] = max[1].max(p[1]);
            }
        }
        [min[0], min[1], max[0], max[1]]
    }

    /// Moves and scales this polygon into the unit square, centered on the longer axis.
    ///
    /// This is Android's `normalized()`: a uniform scale, so an oval stays an oval instead of
    /// being stretched into a circle.
    pub fn normalized(&self) -> RoundedPolygon {
        let [left, top, right, bottom] = self.bounds();
        let (width, height) = (right - left, bottom - top);
        let side = width.max(height).max(1e-6);
        let offset = [(side - width) / 2.0 - left, (side - height) / 2.0 - top];
        self.transformed(|p| [(p[0] + offset[0]) / side, (p[1] + offset[1]) / side])
    }

    /// Maps every anchor, control point and the center.
    pub fn transformed(&self, f: impl Fn([f32; 2]) -> [f32; 2]) -> RoundedPolygon {
        let features = self
            .features
            .iter()
            .map(|feature| Feature {
                kind: feature.kind,
                cubics: feature
                    .cubics
                    .iter()
                    .map(|c| Cubic::new(f(c.anchor0), f(c.control0), f(c.control1), f(c.anchor1)))
                    .collect(),
            })
            .collect();
        RoundedPolygon::from_features(features, f(self.center))
    }

    /// Scales around the origin.
    pub fn scaled(&self, sx: f32, sy: f32) -> RoundedPolygon {
        self.transformed(|p| [p[0] * sx, p[1] * sy])
    }

    /// Rotates `radians` around the origin, matching Android's `Matrix.rotateZ`.
    pub fn rotated(&self, radians: f32) -> RoundedPolygon {
        let (sin, cos) = radians.sin_cos();
        self.transformed(|p| [p[0] * cos - p[1] * sin, p[0] * sin + p[1] * cos])
    }

    /// The outline as a closed polyline in this polygon's coordinates.
    ///
    /// Cubics are flattened adaptively until their controls are within `tolerance` of their chords.
    /// The first point is not repeated at the end; the contour closes implicitly.
    pub fn outline(&self, tolerance: f32) -> Vec<[f32; 2]> {
        assert!(
            !self.cubics.is_empty(),
            "a RoundedPolygon has at least one cubic"
        );
        let tolerance = tolerance.max(1e-5);
        let first = self.cubics[0].anchor0;
        let mut points = vec![first];
        for cubic in &self.cubics {
            cubic.flatten_into(tolerance, 0, &mut points);
        }
        // Drop consecutive duplicates and the closing point. A shape can end with a nearly
        // zero-length edge, so trim every trailing point that sits on the seam, not just one.
        let mut out: Vec<[f32; 2]> = Vec::with_capacity(points.len());
        for p in points {
            if out.last().is_none_or(|last| length(sub(*last, p)) > 1e-6) {
                out.push(p);
            }
        }
        while out.len() > 1 && length(sub(out[out.len() - 1], out[0])) < 1e-6 {
            out.pop();
        }
        out
    }
}

/// The geometry of one rounded vertex, kept between the two passes Android uses: deciding the cut,
/// then building the curves with the cut the adjacent sides allow.
struct RoundedCorner {
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    d1: [f32; 2],
    d2: [f32; 2],
    corner_radius: f32,
    smoothing: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    fn new(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], rounding: CornerRounding) -> RoundedCorner {
        let v01 = sub(p0, p1);
        let v21 = sub(p2, p1);
        let d01 = length(v01);
        let d21 = length(v21);
        if d01 > 0.0 && d21 > 0.0 {
            let d1 = mul(v01, 1.0 / d01);
            let d2 = mul(v21, 1.0 / d21);
            let cos_angle = dot(d1, d2);
            let sin_angle = (1.0 - cos_angle * cos_angle).max(0.0).sqrt();
            let expected_round_cut = if sin_angle > 1e-3 {
                rounding.radius * (cos_angle + 1.0) / sin_angle
            } else {
                0.0
            };
            RoundedCorner {
                p0,
                p1,
                p2,
                d1,
                d2,
                corner_radius: rounding.radius,
                smoothing: rounding.smoothing,
                expected_round_cut,
            }
        } else {
            RoundedCorner {
                p0,
                p1,
                p2,
                d1: [0.0, 0.0],
                d2: [0.0, 0.0],
                corner_radius: 0.0,
                smoothing: 0.0,
                expected_round_cut: 0.0,
            }
        }
    }

    /// The cut before smoothing: `0` is no cut, `1` doubles it.
    fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    fn actual_smoothing(&self, allowed_cut: f32) -> f32 {
        if allowed_cut > self.expected_cut() {
            self.smoothing
        } else if allowed_cut > self.expected_round_cut {
            self.smoothing * (allowed_cut - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        }
    }

    fn cubics(&self, allowed0: f32, allowed1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed0.min(allowed1);
        if self.expected_round_cut < DISTANCE_EPSILON
            || allowed_cut < DISTANCE_EPSILON
            || self.corner_radius < DISTANCE_EPSILON
        {
            return vec![Cubic::straight_line(self.p1, self.p1)];
        }
        let actual_round_cut = allowed_cut.min(self.expected_round_cut);
        let actual_smoothing0 = self.actual_smoothing(allowed0);
        let actual_smoothing1 = self.actual_smoothing(allowed1);
        let actual_radius = self.corner_radius * actual_round_cut / self.expected_round_cut;
        let center_distance =
            (actual_radius * actual_radius + actual_round_cut * actual_round_cut).sqrt();
        let Some(bisector) = direction(mul(add(self.d1, self.d2), 0.5)) else {
            return vec![Cubic::straight_line(self.p1, self.p1)];
        };
        let center = add(self.p1, mul(bisector, center_distance));
        let circle_intersection0 = add(self.p1, mul(self.d1, actual_round_cut));
        let circle_intersection2 = add(self.p1, mul(self.d2, actual_round_cut));
        let flanking0 = self.flanking_curve(
            actual_round_cut,
            actual_smoothing0,
            self.p0,
            circle_intersection0,
            circle_intersection2,
            center,
            actual_radius,
        );
        let flanking2 = self
            .flanking_curve(
                actual_round_cut,
                actual_smoothing1,
                self.p2,
                circle_intersection2,
                circle_intersection0,
                center,
                actual_radius,
            )
            .reversed();
        vec![
            flanking0,
            Cubic::circular_arc(center, flanking0.anchor1, flanking2.anchor0),
            flanking2,
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn flanking_curve(
        &self,
        actual_round_cut: f32,
        smoothing: f32,
        side_start: [f32; 2],
        circle_intersection: [f32; 2],
        other_intersection: [f32; 2],
        circle_center: [f32; 2],
        actual_radius: f32,
    ) -> Cubic {
        let Some(side_direction) = direction(sub(side_start, self.p1)) else {
            return Cubic::straight_line(self.p1, circle_intersection);
        };
        let curve_start = add(
            self.p1,
            mul(side_direction, actual_round_cut * (1.0 + smoothing)),
        );
        let p = lerp(
            circle_intersection,
            mul(add(circle_intersection, other_intersection), 0.5),
            smoothing,
        );
        let curve_end = add(
            circle_center,
            mul(
                direction(sub(p, circle_center)).unwrap_or([1.0, 0.0]),
                actual_radius,
            ),
        );
        let circle_tangent = rotate90(sub(curve_end, circle_center));
        let anchor_end = line_intersection(side_start, side_direction, curve_end, circle_tangent)
            .unwrap_or(circle_intersection);
        let anchor_start = mul(add(curve_start, mul(anchor_end, 2.0)), 1.0 / 3.0);
        Cubic::new(curve_start, anchor_start, anchor_end, curve_end)
    }
}

/// The intersection of the lines through `p0` in direction `d0` and through `p1` in direction `d1`.
fn line_intersection(p0: [f32; 2], d0: [f32; 2], p1: [f32; 2], d1: [f32; 2]) -> Option<[f32; 2]> {
    let rotated_d1 = rotate90(d1);
    let den = dot(d0, rotated_d1);
    if den.abs() < DISTANCE_EPSILON {
        return None;
    }
    let num = dot(sub(p1, p0), rotated_d1);
    if den.abs() < DISTANCE_EPSILON * num.abs() {
        return None;
    }
    Some(add(p0, mul(d0, num / den)))
}
