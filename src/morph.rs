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
// Ported from androidx.graphics.shapes.Morph, modified here; see README.md.

//! Shape morphing: Android's `androidx.graphics.shapes.Morph`.
//!
//! Two [`RoundedPolygon`]s rarely have the same cubics: a circle has ten rounded corners and a
//! heart has four, a burst has two dozen, and even shapes with the same count start at different
//! points. [`Morph`] measures where every corner and cubic sits along each outline, maps corners
//! to the closest compatible corner (convex to convex, concave to concave), cuts the longer curves
//! where the mappings cross, and pairs the resulting cubics one to one. Interpolating those pairs
//! gives a smooth in-between shape, not a remesh of two arbitrary paths.
//!
//! This is a port of Android's algorithm, including its `LengthMeasurer`, `MeasuredPolygon`,
//! `DoubleMapper` and feature mapper.
//!
//! ```
//! use rune_material_shapes::{MaterialShape, Morph};
//!
//! let circle = MaterialShape::Circle.polygon();
//! let heart = MaterialShape::Heart.polygon();
//! let morph = Morph::new(circle, heart);
//! let middle = morph.polygon(0.5);
//! assert!(middle.outline(0.002).len() > 8);
//! ```

use crate::geometry::{Cubic, FeatureKind, RoundedPolygon};

const ANGLE_EPSILON: f32 = 1e-6;
const DISTANCE_EPSILON: f32 = 1e-4;
/// `LengthMeasurer` splits each cubic into this many segments.
const LENGTH_SEGMENTS: usize = 3;

/// A shape animating from `start` to `end`.
///
/// Build it once, then ask for the shape at any progress. Values outside `0..=1` extrapolate.
#[derive(Clone, Debug)]
pub struct Morph {
    start: RoundedPolygon,
    end: RoundedPolygon,
    pairs: Vec<(Cubic, Cubic)>,
}

impl Morph {
    /// Matches the two polygons' curves. This is the expensive part; building the shape at a
    /// progress afterwards is a linear interpolation per pair.
    pub fn new(start: RoundedPolygon, end: RoundedPolygon) -> Morph {
        let pairs = match_polygons(&start, &end);
        Morph { start, end, pairs }
    }

    /// The shape the morph starts from.
    pub fn start(&self) -> &RoundedPolygon {
        &self.start
    }

    /// The shape the morph ends at.
    pub fn end(&self) -> &RoundedPolygon {
        &self.end
    }

    /// How many cubic pairs the match produced.
    pub fn curve_count(&self) -> usize {
        self.pairs.len()
    }

    /// The morph at `progress`: 0 is the start shape, 1 the end shape.
    pub fn polygon(&self, progress: f32) -> RoundedPolygon {
        let cubics = self.cubics(progress);
        let center = average_anchors(&cubics);
        RoundedPolygon::from_cubics(cubics, center)
    }

    /// The morph's outline at `progress`, flattened to `tolerance`, for painting.
    pub fn outline(&self, progress: f32, tolerance: f32) -> Vec<[f32; 2]> {
        self.polygon(progress).outline(tolerance)
    }

    /// The union of the start and end bounds.
    pub fn bounds(&self) -> [f32; 4] {
        let a = self.start.bounds();
        let b = self.end.bounds();
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    }

    /// The paired cubics interpolated at `progress`, with the last one's end joined exactly to the
    /// first one's start as Android does.
    fn cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut out = Vec::with_capacity(self.pairs.len());
        let mut first: Option<Cubic> = None;
        let mut last: Option<Cubic> = None;
        for (a, b) in &self.pairs {
            let cubic = Cubic::new(
                lerp(a.anchor0, b.anchor0, progress),
                lerp(a.control0, b.control0, progress),
                lerp(a.control1, b.control1, progress),
                lerp(a.anchor1, b.anchor1, progress),
            );
            if first.is_none() {
                first = Some(cubic);
            }
            if let Some(previous) = last {
                out.push(previous);
            }
            last = Some(cubic);
        }
        if let (Some(last), Some(first)) = (last, first) {
            out.push(Cubic::new(
                last.anchor0,
                last.control0,
                last.control1,
                first.anchor0,
            ));
        }
        out
    }
}

fn average_anchors(cubics: &[Cubic]) -> [f32; 2] {
    if cubics.is_empty() {
        return [0.5, 0.5];
    }
    let mut sum = [0.0, 0.0];
    for c in cubics {
        sum[0] += c.anchor0[0];
        sum[1] += c.anchor0[1];
    }
    let n = cubics.len() as f32;
    [sum[0] / n, sum[1] / n]
}

/// One cubic with its share of the outline, and where that share starts and ends.
#[derive(Clone, Debug)]
struct MeasuredCubic {
    cubic: Cubic,
    start: f32,
    end: f32,
    size: f32,
}

impl MeasuredCubic {
    fn new(cubic: Cubic, start: f32, end: f32) -> MeasuredCubic {
        MeasuredCubic {
            cubic,
            start,
            end,
            size: measure_cubic(cubic),
        }
    }

    /// Android's `cutAtProgress`: cuts at an outline progress in this cubic's range.
    fn cut_at_progress(&self, cut: f32) -> (MeasuredCubic, MeasuredCubic) {
        let bounded = cut.clamp(self.start, self.end);
        let span = self.end - self.start;
        let relative = if span > 0.0 {
            (bounded - self.start) / span
        } else {
            0.0
        };
        let t = find_cubic_cut_point(self.cubic, relative * self.size).clamp(0.0, 1.0);
        let (a, b) = self.cubic.split(t);
        (
            MeasuredCubic::new(a, self.start, bounded),
            MeasuredCubic::new(b, bounded, self.end),
        )
    }
}

/// A corner with its progress along the outline.
#[derive(Clone, Debug)]
struct MeasuredFeature {
    progress: f32,
    convex: bool,
    point: [f32; 2],
}

/// A polygon whose cubics carry outline progress, plus its corners.
#[derive(Clone, Debug)]
struct MeasuredPolygon {
    cubics: Vec<MeasuredCubic>,
    features: Vec<MeasuredFeature>,
}

impl MeasuredPolygon {
    fn new(
        features: Vec<MeasuredFeature>,
        cubics: Vec<Cubic>,
        outline: Vec<f32>,
    ) -> MeasuredPolygon {
        assert_eq!(
            outline.len(),
            cubics.len() + 1,
            "one more outline progress than cubics"
        );
        assert_eq!(outline.first(), Some(&0.0));
        assert_eq!(outline.last(), Some(&1.0));
        let mut measured = Vec::with_capacity(cubics.len());
        let mut start = 0.0f32;
        for (i, cubic) in cubics.into_iter().enumerate() {
            if outline[i + 1] - outline[i] > DISTANCE_EPSILON {
                measured.push(MeasuredCubic::new(cubic, start, outline[i + 1]));
                start = outline[i + 1];
            }
        }
        measured
            .last_mut()
            .expect("a measured polygon has cubics")
            .end = 1.0;
        MeasuredPolygon {
            cubics: measured,
            features,
        }
    }

    /// Android's `MeasuredPolygon.measurePolygon(LengthMeasurer(), polygon)`.
    fn measure(polygon: &RoundedPolygon) -> MeasuredPolygon {
        let mut cubics = Vec::new();
        let mut feature_to_cubic: Vec<(bool, [f32; 2], usize)> = Vec::new();
        for feature in polygon.features() {
            for (cubic_index, cubic) in feature.cubics.iter().enumerate() {
                if feature.kind != FeatureKind::Edge && cubic_index == feature.cubics.len() / 2 {
                    feature_to_cubic.push((
                        feature.kind == FeatureKind::ConvexCorner,
                        feature.representative_point(),
                        cubics.len(),
                    ));
                }
                cubics.push(*cubic);
            }
        }

        let mut measures = Vec::with_capacity(cubics.len() + 1);
        measures.push(0.0f32);
        let mut total = 0.0f32;
        for cubic in &cubics {
            let size = measure_cubic(*cubic);
            assert!(size >= 0.0, "a measured cubic is not negative");
            total += size;
            measures.push(total);
        }
        let total = total.max(f32::MIN_POSITIVE);
        let outline: Vec<f32> = measures.iter().map(|m| m / total).collect();

        let features = feature_to_cubic
            .into_iter()
            .map(|(convex, point, index)| MeasuredFeature {
                progress: positive_modulo((outline[index] + outline[index + 1]) / 2.0, 1.0),
                convex,
                point,
            })
            .collect();
        MeasuredPolygon::new(features, cubics, outline)
    }

    /// Android's `cutAndShift`: rotate the contour so `cut` becomes progress 0.
    fn cut_and_shift(&self, cut: f32) -> MeasuredPolygon {
        assert!((0.0..=1.0).contains(&cut), "cut point is in 0..=1");
        if cut < DISTANCE_EPSILON {
            return self.clone();
        }
        let target_index = self
            .cubics
            .iter()
            .position(|c| cut >= c.start && cut <= c.end)
            .expect("the cut point lies on a cubic");
        let target = &self.cubics[target_index];
        let (before, after) = target.cut_at_progress(cut);

        let mut cubics = Vec::with_capacity(self.cubics.len() + 1);
        cubics.push(after.cubic);
        for i in 1..self.cubics.len() {
            cubics.push(self.cubics[(i + target_index) % self.cubics.len()].cubic);
        }
        cubics.push(before.cubic);

        let mut outline = Vec::with_capacity(cubics.len() + 1);
        for index in 0..self.cubics.len() + 2 {
            match index {
                0 => outline.push(0.0),
                n if n == self.cubics.len() + 1 => outline.push(1.0),
                _ => {
                    let cubic = &self.cubics[(target_index + index - 1) % self.cubics.len()];
                    // Compute the wrap in f64: in f32 a value just below 1.0 rounds to 1.0
                    // and then back to 0.0, which would drop the cubic before the cut.
                    let shifted = f64::from(cubic.end) - f64::from(cut);
                    outline.push((((shifted % 1.0) + 1.0) % 1.0) as f32);
                }
            }
        }

        let features = self
            .features
            .iter()
            .map(|f| MeasuredFeature {
                progress: positive_modulo(f.progress - cut, 1.0),
                ..f.clone()
            })
            .collect();
        MeasuredPolygon::new(features, cubics, outline)
    }
}

/// Android's `Morph.match`.
fn match_polygons(start: &RoundedPolygon, end: &RoundedPolygon) -> Vec<(Cubic, Cubic)> {
    let measured_start = MeasuredPolygon::measure(start);
    let measured_end = MeasuredPolygon::measure(end);
    let mapper = feature_mapper(&measured_start.features, &measured_end.features);
    let cut = mapper.map(0.0);
    let start_polygon = measured_start;
    let end_polygon = measured_end.cut_and_shift(cut);

    let mut pairs = Vec::new();
    let mut i1 = 0;
    let mut i2 = 0;
    let mut b1 = start_polygon.cubics.get(i1).cloned();
    i1 += 1;
    let mut b2 = end_polygon.cubics.get(i2).cloned();
    i2 += 1;
    while let (Some(current1), Some(current2)) = (b1.clone(), b2.clone()) {
        let b1a = if i1 == start_polygon.cubics.len() {
            1.0
        } else {
            current1.end
        };
        let b2a = if i2 == end_polygon.cubics.len() {
            1.0
        } else {
            mapper.map_back(positive_modulo(current2.end + cut, 1.0))
        };
        let minb = b1a.min(b2a);

        let (seg1, next1) = if b1a > minb + ANGLE_EPSILON {
            let (seg, next) = current1.cut_at_progress(minb);
            (seg, Some(next))
        } else {
            (current1, start_polygon.cubics.get(i1).cloned())
        };
        if b1a <= minb + ANGLE_EPSILON {
            i1 += 1;
        }

        let (seg2, next2) = if b2a > minb + ANGLE_EPSILON {
            let cut2 = positive_modulo(mapper.map(minb) - cut, 1.0);
            let (seg, next) = current2.cut_at_progress(cut2);
            (seg, Some(next))
        } else {
            (current2, end_polygon.cubics.get(i2).cloned())
        };
        if b2a <= minb + ANGLE_EPSILON {
            i2 += 1;
        }

        pairs.push((seg1.cubic, seg2.cubic));
        b1 = next1;
        b2 = next2;
    }
    assert!(
        b1.is_none() && b2.is_none(),
        "both polygons' cubics are fully matched"
    );
    pairs
}

/// Android's `featureMapper`: maps only corners, then extends those anchors to the whole outline.
fn feature_mapper(start: &[MeasuredFeature], end: &[MeasuredFeature]) -> DoubleMapper {
    let mapping = do_mapping(start, end);
    DoubleMapper::new(&mapping)
}

#[derive(Clone, Copy)]
struct DistanceVertex {
    distance: f32,
    start: usize,
    end: usize,
}

/// Android's `doMapping`: greedily match the closest corners that keep the mapping monotonic.
fn do_mapping(start: &[MeasuredFeature], end: &[MeasuredFeature]) -> Vec<(f32, f32)> {
    let mut distances = Vec::new();
    for (i, f1) in start.iter().enumerate() {
        for (j, f2) in end.iter().enumerate() {
            let distance = feature_distance_squared(f1, f2);
            if distance != f32::MAX {
                distances.push(DistanceVertex {
                    distance,
                    start: i,
                    end: j,
                });
            }
        }
    }
    distances.sort_by(|a, b| a.distance.total_cmp(&b.distance));

    if distances.is_empty() {
        return vec![(0.0, 0.0), (0.5, 0.5)];
    }
    if distances.len() == 1 {
        let f1 = start[distances[0].start].progress;
        let f2 = end[distances[0].end].progress;
        return vec![(f1, f2), ((f1 + 0.5) % 1.0, (f2 + 0.5) % 1.0)];
    }

    let mut helper = MappingHelper {
        mapping: Vec::new(),
        used_start: vec![false; start.len()],
        used_end: vec![false; end.len()],
    };
    for vertex in distances {
        helper.add(start, end, vertex.start, vertex.end);
    }
    helper.mapping
}

fn feature_distance_squared(f1: &MeasuredFeature, f2: &MeasuredFeature) -> f32 {
    if f1.convex != f2.convex {
        return f32::MAX;
    }
    let dx = f1.point[0] - f2.point[0];
    let dy = f1.point[1] - f2.point[1];
    dx * dx + dy * dy
}

/// Android's `MappingHelper`.
struct MappingHelper {
    mapping: Vec<(f32, f32)>,
    used_start: Vec<bool>,
    used_end: Vec<bool>,
}

impl MappingHelper {
    fn add(&mut self, start: &[MeasuredFeature], end: &[MeasuredFeature], i1: usize, i2: usize) {
        if self.used_start[i1] || self.used_end[i2] {
            return;
        }
        let f1 = start[i1].progress;
        let f2 = end[i2].progress;
        let insertion = match self
            .mapping
            .binary_search_by(|(progress, _)| progress.total_cmp(&f1))
        {
            Ok(_) => return,
            Err(index) => index,
        };
        let n = self.mapping.len();
        if n >= 1 {
            let (before1, before2) = self.mapping[(insertion + n - 1) % n];
            let (after1, after2) = self.mapping[insertion % n];
            if progress_distance(f1, before1) < DISTANCE_EPSILON
                || progress_distance(f1, after1) < DISTANCE_EPSILON
                || progress_distance(f2, before2) < DISTANCE_EPSILON
                || progress_distance(f2, after2) < DISTANCE_EPSILON
            {
                return;
            }
            if n > 1 && !progress_in_range(f2, before2, after2) {
                return;
            }
        }
        self.mapping.insert(insertion, (f1, f2));
        self.used_start[i1] = true;
        self.used_end[i2] = true;
    }
}

/// Android's `DoubleMapper`: mappings between the two outlines' progress spaces.
#[derive(Clone, Debug)]
struct DoubleMapper {
    source: Vec<f32>,
    target: Vec<f32>,
}

impl DoubleMapper {
    fn new(mappings: &[(f32, f32)]) -> DoubleMapper {
        let source: Vec<f32> = mappings.iter().map(|(a, _)| *a).collect();
        let target: Vec<f32> = mappings.iter().map(|(_, b)| *b).collect();
        validate_progress(&source);
        validate_progress(&target);
        DoubleMapper { source, target }
    }

    fn map(&self, x: f32) -> f32 {
        linear_map(&self.source, &self.target, x)
    }

    fn map_back(&self, x: f32) -> f32 {
        linear_map(&self.target, &self.source, x)
    }
}

/// Android's `linearMap`: piecewise linear with wrap-around.
fn linear_map(x_values: &[f32], y_values: &[f32], x: f32) -> f32 {
    assert!((0.0..=1.0).contains(&x), "progress is in 0..=1: {x}");
    let n = x_values.len();
    let segment = (0..n)
        .find(|&i| progress_in_range(x, x_values[i], x_values[(i + 1) % n]))
        .expect("every progress lies in a segment");
    let end = (segment + 1) % n;
    let size_x = positive_modulo(x_values[end] - x_values[segment], 1.0);
    let size_y = positive_modulo(y_values[end] - y_values[segment], 1.0);
    let position = if size_x < 0.001 {
        0.5
    } else {
        positive_modulo(x - x_values[segment], 1.0) / size_x
    };
    positive_modulo(y_values[segment] + size_y * position, 1.0)
}

fn validate_progress(values: &[f32]) {
    let mut previous = *values.last().expect("at least one mapping");
    let mut wraps = 0;
    for &current in values {
        assert!(
            (0.0..1.0).contains(&current),
            "progress is in 0..1: {values:?}"
        );
        assert!(
            progress_distance(current, previous) > DISTANCE_EPSILON,
            "progress repeats a value: {values:?}"
        );
        if current < previous {
            wraps += 1;
            assert!(wraps <= 1, "progress wraps more than once: {values:?}");
        }
        previous = current;
    }
}

fn progress_in_range(progress: f32, from: f32, to: f32) -> bool {
    if to >= from {
        (from..=to).contains(&progress)
    } else {
        progress >= from || progress <= to
    }
}

fn progress_distance(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs();
    distance.min(1.0 - distance)
}

fn positive_modulo(value: f32, modulus: f32) -> f32 {
    (value % modulus + modulus) % modulus
}

/// Android's `LengthMeasurer`: approximate a cubic's arc length with three chords, and find the
/// parameter at a given length along them.
fn measure_cubic(cubic: Cubic) -> f32 {
    closest_progress_to(cubic, f32::INFINITY).1
}

fn find_cubic_cut_point(cubic: Cubic, measure: f32) -> f32 {
    closest_progress_to(cubic, measure).0
}

fn closest_progress_to(cubic: Cubic, threshold: f32) -> (f32, f32) {
    let mut total = 0.0f32;
    let mut remainder = threshold;
    let mut previous = cubic.anchor0;
    for i in 1..=LENGTH_SEGMENTS {
        let progress = i as f32 / LENGTH_SEGMENTS as f32;
        let point = cubic.point_at(progress);
        let segment = distance(point, previous);
        if segment >= remainder {
            return (
                progress - (1.0 - remainder / segment) / LENGTH_SEGMENTS as f32,
                threshold,
            );
        }
        remainder -= segment;
        total += segment;
        previous = point;
    }
    (1.0, total)
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
