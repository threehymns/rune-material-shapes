// Copyright 2024 The Android Open Source Project
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
// Ported from androidx.compose.material3.MaterialShapes, modified here; see README.md.

//! The 35 named shapes of Material 3 Expressive's shape library.
//!
//! Every variant is the geometry of `androidx.compose.material3.MaterialShapes`: the same vertices,
//! corner radii, smoothing and transforms, normalized into the unit square. [`MaterialShape::ALL`]
//! lists them in Android's order, and [`MaterialShape::polygon`] builds one.

use std::f32::consts::PI;

use crate::geometry::{CornerRounding, RoundedPolygon};

/// The groups the Material shape library is browsed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Circles, squares, triangles and the other everyday outlines.
    Basic,
    /// Circular outlines scalloped like a cookie.
    Cookies,
    /// Lobed, clover-like outlines.
    Clovers,
    /// Spiked, spark-like outlines.
    Bursts,
    /// One-off shapes with their own character.
    Unique,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Basic => "Basic",
            Category::Cookies => "Cookies",
            Category::Clovers => "Clovers",
            Category::Bursts => "Bursts",
            Category::Unique => "Unique",
        }
    }
}

/// Android's name for the catalog: `MaterialShapes::Circle` is `MaterialShapes.Circle`.
pub type MaterialShapes = MaterialShape;

/// One shape from Material 3 Expressive's shape library.
///
/// In Kotlin these are the `MaterialShapes` properties (`MaterialShapes.Cookie12Sided`); here the
/// enum is the catalog, and `MaterialShape::Cookie12Sided.polygon()` is the `RoundedPolygon`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialShape {
    Circle,
    Square,
    Slanted,
    Arch,
    Fan,
    Arrow,
    SemiCircle,
    Oval,
    Pill,
    Triangle,
    Diamond,
    ClamShell,
    Pentagon,
    Gem,
    Sunny,
    VerySunny,
    Cookie4Sided,
    Cookie6Sided,
    Cookie7Sided,
    Cookie9Sided,
    Cookie12Sided,
    Ghostish,
    Clover4Leaf,
    Clover8Leaf,
    Burst,
    SoftBurst,
    Boom,
    SoftBoom,
    Flower,
    Puffy,
    PuffyDiamond,
    PixelCircle,
    PixelTriangle,
    Bun,
    Heart,
}

impl MaterialShape {
    /// Every shape, in Android's `MaterialShapes` order.
    pub const ALL: [MaterialShape; 35] = [
        MaterialShape::Circle,
        MaterialShape::Square,
        MaterialShape::Slanted,
        MaterialShape::Arch,
        MaterialShape::Fan,
        MaterialShape::Arrow,
        MaterialShape::SemiCircle,
        MaterialShape::Oval,
        MaterialShape::Pill,
        MaterialShape::Triangle,
        MaterialShape::Diamond,
        MaterialShape::ClamShell,
        MaterialShape::Pentagon,
        MaterialShape::Gem,
        MaterialShape::Sunny,
        MaterialShape::VerySunny,
        MaterialShape::Cookie4Sided,
        MaterialShape::Cookie6Sided,
        MaterialShape::Cookie7Sided,
        MaterialShape::Cookie9Sided,
        MaterialShape::Cookie12Sided,
        MaterialShape::Ghostish,
        MaterialShape::Clover4Leaf,
        MaterialShape::Clover8Leaf,
        MaterialShape::Burst,
        MaterialShape::SoftBurst,
        MaterialShape::Boom,
        MaterialShape::SoftBoom,
        MaterialShape::Flower,
        MaterialShape::Puffy,
        MaterialShape::PuffyDiamond,
        MaterialShape::PixelCircle,
        MaterialShape::PixelTriangle,
        MaterialShape::Bun,
        MaterialShape::Heart,
    ];

    /// The Compose property's name, such as `"Cookie12Sided"`.
    pub fn name(self) -> &'static str {
        match self {
            MaterialShape::Circle => "Circle",
            MaterialShape::Square => "Square",
            MaterialShape::Slanted => "Slanted",
            MaterialShape::Arch => "Arch",
            MaterialShape::Fan => "Fan",
            MaterialShape::Arrow => "Arrow",
            MaterialShape::SemiCircle => "SemiCircle",
            MaterialShape::Oval => "Oval",
            MaterialShape::Pill => "Pill",
            MaterialShape::Triangle => "Triangle",
            MaterialShape::Diamond => "Diamond",
            MaterialShape::ClamShell => "ClamShell",
            MaterialShape::Pentagon => "Pentagon",
            MaterialShape::Gem => "Gem",
            MaterialShape::Sunny => "Sunny",
            MaterialShape::VerySunny => "VerySunny",
            MaterialShape::Cookie4Sided => "Cookie4Sided",
            MaterialShape::Cookie6Sided => "Cookie6Sided",
            MaterialShape::Cookie7Sided => "Cookie7Sided",
            MaterialShape::Cookie9Sided => "Cookie9Sided",
            MaterialShape::Cookie12Sided => "Cookie12Sided",
            MaterialShape::Ghostish => "Ghostish",
            MaterialShape::Clover4Leaf => "Clover4Leaf",
            MaterialShape::Clover8Leaf => "Clover8Leaf",
            MaterialShape::Burst => "Burst",
            MaterialShape::SoftBurst => "SoftBurst",
            MaterialShape::Boom => "Boom",
            MaterialShape::SoftBoom => "SoftBoom",
            MaterialShape::Flower => "Flower",
            MaterialShape::Puffy => "Puffy",
            MaterialShape::PuffyDiamond => "PuffyDiamond",
            MaterialShape::PixelCircle => "PixelCircle",
            MaterialShape::PixelTriangle => "PixelTriangle",
            MaterialShape::Bun => "Bun",
            MaterialShape::Heart => "Heart",
        }
    }

    /// The shape's name for people, such as `"Cookie 12-sided"`.
    pub fn label(self) -> &'static str {
        match self {
            MaterialShape::Circle => "Circle",
            MaterialShape::Square => "Square",
            MaterialShape::Slanted => "Slanted",
            MaterialShape::Arch => "Arch",
            MaterialShape::Fan => "Fan",
            MaterialShape::Arrow => "Arrow",
            MaterialShape::SemiCircle => "Semi-circle",
            MaterialShape::Oval => "Oval",
            MaterialShape::Pill => "Pill",
            MaterialShape::Triangle => "Triangle",
            MaterialShape::Diamond => "Diamond",
            MaterialShape::ClamShell => "Clam shell",
            MaterialShape::Pentagon => "Pentagon",
            MaterialShape::Gem => "Gem",
            MaterialShape::Sunny => "Sunny",
            MaterialShape::VerySunny => "Very sunny",
            MaterialShape::Cookie4Sided => "Cookie 4-sided",
            MaterialShape::Cookie6Sided => "Cookie 6-sided",
            MaterialShape::Cookie7Sided => "Cookie 7-sided",
            MaterialShape::Cookie9Sided => "Cookie 9-sided",
            MaterialShape::Cookie12Sided => "Cookie 12-sided",
            MaterialShape::Ghostish => "Ghostish",
            MaterialShape::Clover4Leaf => "Clover 4-leaf",
            MaterialShape::Clover8Leaf => "Clover 8-leaf",
            MaterialShape::Burst => "Burst",
            MaterialShape::SoftBurst => "Soft burst",
            MaterialShape::Boom => "Boom",
            MaterialShape::SoftBoom => "Soft boom",
            MaterialShape::Flower => "Flower",
            MaterialShape::Puffy => "Puffy",
            MaterialShape::PuffyDiamond => "Puffy diamond",
            MaterialShape::PixelCircle => "Pixel circle",
            MaterialShape::PixelTriangle => "Pixel triangle",
            MaterialShape::Bun => "Bun",
            MaterialShape::Heart => "Heart",
        }
    }

    /// The group this shape belongs to.
    pub fn category(self) -> Category {
        use MaterialShape::*;
        match self {
            Circle | Square | Slanted | Arch | Fan | Arrow | SemiCircle | Oval | Pill
            | Triangle | Diamond | ClamShell | Pentagon | Gem => Category::Basic,
            Sunny | VerySunny | Cookie4Sided | Cookie6Sided | Cookie7Sided | Cookie9Sided
            | Cookie12Sided => Category::Cookies,
            Ghostish | Clover4Leaf | Clover8Leaf => Category::Clovers,
            Burst | SoftBurst | Boom | SoftBoom | Flower => Category::Bursts,
            Puffy | PuffyDiamond | PixelCircle | PixelTriangle | Bun | Heart => Category::Unique,
        }
    }

    /// Every shape in `category`, in library order.
    pub fn in_category(category: Category) -> impl Iterator<Item = MaterialShape> {
        MaterialShape::ALL
            .into_iter()
            .filter(move |s| s.category() == category)
    }

    /// This shape as a normalized [`RoundedPolygon`]: the outline fills the unit square on its
    /// longer axis and is centered on the other.
    pub fn polygon(self) -> RoundedPolygon {
        use MaterialShape::*;
        let polygon = match self {
            Circle => RoundedPolygon::circle(10, 1.0),
            Square => RoundedPolygon::rectangle(1.0, 1.0, r30()),
            Slanted => custom(
                &[
                    ([0.926, 0.970], CornerRounding::smoothed(0.189, 0.811)),
                    ([-0.021, 0.967], CornerRounding::smoothed(0.187, 0.057)),
                ],
                2,
                false,
            ),
            Arch => {
                RoundedPolygon::regular_with(4, 1.0, [0.0, 0.0], &[r100(), r100(), r20(), r20()])
                    .rotated(-135.0f32.to_radians())
            }
            Fan => custom(
                &[
                    ([1.004, 1.000], CornerRounding::smoothed(0.148, 0.417)),
                    ([0.000, 1.000], CornerRounding::smoothed(0.151, 0.0)),
                    ([0.000, -0.003], CornerRounding::smoothed(0.148, 0.0)),
                    ([0.978, 0.020], CornerRounding::smoothed(0.803, 0.0)),
                ],
                1,
                false,
            ),
            Arrow => custom(
                &[
                    ([0.500, 0.892], CornerRounding::smoothed(0.313, 0.0)),
                    ([-0.216, 1.050], CornerRounding::smoothed(0.207, 0.0)),
                    ([0.499, -0.160], CornerRounding::smoothed(0.215, 1.000)),
                    ([1.225, 1.060], CornerRounding::smoothed(0.211, 0.0)),
                ],
                1,
                false,
            ),
            SemiCircle => RoundedPolygon::rectangle_with(1.6, 1.0, &[r20(), r20(), r100(), r100()]),
            Oval => RoundedPolygon::circle(8, 1.0)
                .scaled(1.0, 0.64)
                .rotated(-45.0f32.to_radians()),
            Pill => custom(
                &[
                    ([0.961, 0.039], CornerRounding::smoothed(0.426, 0.0)),
                    ([1.001, 0.428], CornerRounding::UNROUNDED),
                    ([1.000, 0.609], CornerRounding::smoothed(1.000, 0.0)),
                ],
                2,
                true,
            ),
            Triangle => {
                RoundedPolygon::regular(3, 1.0, [0.0, 0.0], r20()).rotated(-90.0f32.to_radians())
            }
            Diamond => custom(
                &[
                    ([0.500, 1.096], CornerRounding::smoothed(0.151, 0.524)),
                    ([0.040, 0.500], CornerRounding::smoothed(0.159, 0.0)),
                ],
                2,
                false,
            ),
            ClamShell => custom(
                &[
                    ([0.171, 0.841], CornerRounding::smoothed(0.159, 0.0)),
                    ([-0.020, 0.500], CornerRounding::smoothed(0.140, 0.0)),
                    ([0.170, 0.159], CornerRounding::smoothed(0.159, 0.0)),
                ],
                2,
                false,
            ),
            Pentagon => custom(
                &[
                    ([0.500, -0.009], CornerRounding::smoothed(0.172, 0.0)),
                    ([1.030, 0.365], CornerRounding::smoothed(0.164, 0.0)),
                    ([0.828, 0.970], CornerRounding::smoothed(0.169, 0.0)),
                ],
                1,
                true,
            ),
            Gem => custom(
                &[
                    ([0.499, 1.023], CornerRounding::smoothed(0.241, 0.778)),
                    ([-0.005, 0.792], CornerRounding::smoothed(0.208, 0.0)),
                    ([0.073, 0.258], CornerRounding::smoothed(0.228, 0.0)),
                    ([0.433, -0.000], CornerRounding::smoothed(0.491, 0.0)),
                ],
                1,
                true,
            ),
            Sunny => RoundedPolygon::star(8, 1.0, 0.8, r15()),
            VerySunny => custom(
                &[
                    ([0.500, 1.080], CornerRounding::smoothed(0.085, 0.0)),
                    ([0.358, 0.843], CornerRounding::smoothed(0.085, 0.0)),
                ],
                8,
                false,
            ),
            Cookie4Sided => custom(
                &[
                    ([1.237, 1.236], CornerRounding::smoothed(0.258, 0.0)),
                    ([0.500, 0.918], CornerRounding::smoothed(0.233, 0.0)),
                ],
                4,
                false,
            ),
            Cookie6Sided => custom(
                &[
                    ([0.723, 0.884], CornerRounding::smoothed(0.394, 0.0)),
                    ([0.500, 1.099], CornerRounding::smoothed(0.398, 0.0)),
                ],
                6,
                false,
            ),
            Cookie7Sided => {
                RoundedPolygon::star(7, 1.0, 0.75, r50()).rotated(-90.0f32.to_radians())
            }
            Cookie9Sided => RoundedPolygon::star(9, 1.0, 0.8, r50()).rotated(-90.0f32.to_radians()),
            Cookie12Sided => {
                RoundedPolygon::star(12, 1.0, 0.8, r50()).rotated(-90.0f32.to_radians())
            }
            Ghostish => custom(
                &[
                    ([0.500, 0.000], r100()),
                    ([1.000, 0.000], r100()),
                    ([1.000, 1.140], CornerRounding::smoothed(0.254, 0.106)),
                    ([0.575, 0.906], CornerRounding::smoothed(0.253, 0.0)),
                ],
                1,
                true,
            ),
            Clover4Leaf => custom(
                &[
                    ([0.500, 0.074], CornerRounding::UNROUNDED),
                    ([0.725, -0.099], CornerRounding::smoothed(0.476, 0.0)),
                ],
                4,
                true,
            ),
            Clover8Leaf => custom(
                &[
                    ([0.500, 0.036], CornerRounding::UNROUNDED),
                    ([0.758, -0.101], CornerRounding::smoothed(0.209, 0.0)),
                ],
                8,
                false,
            ),
            Burst => custom(
                &[
                    ([0.500, -0.006], CornerRounding::smoothed(0.006, 0.0)),
                    ([0.592, 0.158], CornerRounding::smoothed(0.006, 0.0)),
                ],
                12,
                false,
            ),
            SoftBurst => custom(
                &[
                    ([0.193, 0.277], CornerRounding::smoothed(0.053, 0.0)),
                    ([0.176, 0.055], CornerRounding::smoothed(0.053, 0.0)),
                ],
                10,
                false,
            ),
            Boom => custom(
                &[
                    ([0.457, 0.296], CornerRounding::smoothed(0.007, 0.0)),
                    ([0.500, -0.051], CornerRounding::smoothed(0.007, 0.0)),
                ],
                15,
                false,
            ),
            SoftBoom => custom(
                &[
                    ([0.733, 0.454], CornerRounding::UNROUNDED),
                    ([0.839, 0.437], CornerRounding::smoothed(0.532, 0.0)),
                    ([0.949, 0.449], CornerRounding::smoothed(0.439, 1.000)),
                    ([0.998, 0.478], CornerRounding::smoothed(0.174, 0.0)),
                ],
                16,
                true,
            ),
            Flower => custom(
                &[
                    ([0.370, 0.187], CornerRounding::UNROUNDED),
                    ([0.416, 0.049], CornerRounding::smoothed(0.381, 0.0)),
                    ([0.479, 0.001], CornerRounding::smoothed(0.095, 0.0)),
                ],
                8,
                true,
            ),
            Puffy => custom(
                &[
                    ([0.500, 0.053], CornerRounding::UNROUNDED),
                    ([0.545, -0.040], CornerRounding::smoothed(0.405, 0.0)),
                    ([0.670, -0.035], CornerRounding::smoothed(0.426, 0.0)),
                    ([0.717, 0.066], CornerRounding::smoothed(0.574, 0.0)),
                    ([0.722, 0.128], CornerRounding::UNROUNDED),
                    ([0.777, 0.002], CornerRounding::smoothed(0.360, 0.0)),
                    ([0.914, 0.149], CornerRounding::smoothed(0.660, 0.0)),
                    ([0.926, 0.289], CornerRounding::smoothed(0.660, 0.0)),
                    ([0.881, 0.346], CornerRounding::UNROUNDED),
                    ([0.940, 0.344], CornerRounding::smoothed(0.126, 0.0)),
                    ([1.003, 0.437], CornerRounding::smoothed(0.255, 0.0)),
                ],
                2,
                true,
            )
            .scaled(1.0, 0.742),
            PuffyDiamond => custom(
                &[
                    ([0.870, 0.130], CornerRounding::smoothed(0.146, 0.0)),
                    ([0.818, 0.357], CornerRounding::UNROUNDED),
                    ([1.000, 0.332], CornerRounding::smoothed(0.853, 0.0)),
                ],
                4,
                true,
            ),
            PixelCircle => custom(
                &[
                    ([0.500, 0.000], CornerRounding::UNROUNDED),
                    ([0.704, 0.000], CornerRounding::UNROUNDED),
                    ([0.704, 0.065], CornerRounding::UNROUNDED),
                    ([0.843, 0.065], CornerRounding::UNROUNDED),
                    ([0.843, 0.148], CornerRounding::UNROUNDED),
                    ([0.926, 0.148], CornerRounding::UNROUNDED),
                    ([0.926, 0.296], CornerRounding::UNROUNDED),
                    ([1.000, 0.296], CornerRounding::UNROUNDED),
                ],
                2,
                true,
            ),
            PixelTriangle => custom(
                &[
                    ([0.110, 0.500], CornerRounding::UNROUNDED),
                    ([0.113, 0.000], CornerRounding::UNROUNDED),
                    ([0.287, 0.000], CornerRounding::UNROUNDED),
                    ([0.287, 0.087], CornerRounding::UNROUNDED),
                    ([0.421, 0.087], CornerRounding::UNROUNDED),
                    ([0.421, 0.170], CornerRounding::UNROUNDED),
                    ([0.560, 0.170], CornerRounding::UNROUNDED),
                    ([0.560, 0.265], CornerRounding::UNROUNDED),
                    ([0.674, 0.265], CornerRounding::UNROUNDED),
                    ([0.675, 0.344], CornerRounding::UNROUNDED),
                    ([0.789, 0.344], CornerRounding::UNROUNDED),
                    ([0.789, 0.439], CornerRounding::UNROUNDED),
                    ([0.888, 0.439], CornerRounding::UNROUNDED),
                ],
                1,
                true,
            ),
            Bun => custom(
                &[
                    ([0.796, 0.500], CornerRounding::UNROUNDED),
                    ([0.853, 0.518], r100()),
                    ([0.992, 0.631], r100()),
                    ([0.968, 1.000], r100()),
                ],
                2,
                true,
            ),
            Heart => custom(
                &[
                    ([0.500, 0.268], CornerRounding::smoothed(0.016, 0.0)),
                    ([0.792, -0.066], CornerRounding::smoothed(0.958, 0.0)),
                    ([1.064, 0.276], CornerRounding::smoothed(1.000, 0.0)),
                    ([0.501, 0.946], CornerRounding::smoothed(0.129, 0.0)),
                ],
                1,
                true,
            ),
        };
        polygon.normalized()
    }
}

fn r15() -> CornerRounding {
    CornerRounding::new(0.15)
}

fn r20() -> CornerRounding {
    CornerRounding::new(0.2)
}

fn r30() -> CornerRounding {
    CornerRounding::new(0.3)
}

fn r50() -> CornerRounding {
    CornerRounding::new(0.5)
}

fn r100() -> CornerRounding {
    CornerRounding::new(1.0)
}

/// Android's `customPolygon`: repeats `points` around `(0.5, 0.5)`, mirrored after each half-turn
/// when `mirroring`, and rounds each vertex as listed.
fn custom(points: &[([f32; 2], CornerRounding)], reps: usize, mirroring: bool) -> RoundedPolygon {
    let center = [0.5, 0.5];
    let vertices = repeat(points, reps, center, mirroring);
    let roundings: Vec<CornerRounding> = vertices.iter().map(|(_, r)| *r).collect();
    let points: Vec<[f32; 2]> = vertices.into_iter().map(|(p, _)| p).collect();
    RoundedPolygon::from_vertices_with(&points, &roundings, Some(center))
}

/// Android's `doRepeat`. Mirroring walks each half-turn in reverse (dropping the seam vertex), so
/// shapes such as the pentagon and heart are symmetric; otherwise each repetition is rotated a
/// whole section.
fn repeat(
    points: &[([f32; 2], CornerRounding)],
    reps: usize,
    center: [f32; 2],
    mirroring: bool,
) -> Vec<([f32; 2], CornerRounding)> {
    if mirroring {
        let angles: Vec<f32> = points
            .iter()
            .map(|(p, _)| angle_degrees(sub(*p, center)))
            .collect();
        let distances: Vec<f32> = points
            .iter()
            .map(|(p, _)| length(sub(*p, center)))
            .collect();
        let actual_reps = reps * 2;
        let section_angle = 360.0 / actual_reps as f32;
        let mut out = Vec::with_capacity(actual_reps * points.len());
        for turn in 0..actual_reps {
            for index in 0..points.len() {
                let i = if turn % 2 == 0 {
                    index
                } else {
                    points.len() - 1 - index
                };
                if i > 0 || turn % 2 == 0 {
                    let a = section_angle * turn as f32
                        + if turn % 2 == 0 {
                            angles[i]
                        } else {
                            section_angle - angles[i] + 2.0 * angles[0]
                        };
                    let radians = a / 360.0 * 2.0 * PI;
                    out.push((
                        [
                            radians.cos() * distances[i] + center[0],
                            radians.sin() * distances[i] + center[1],
                        ],
                        points[i].1,
                    ));
                }
            }
        }
        out
    } else {
        let n = points.len();
        (0..n * reps)
            .map(|i| {
                let angle = (i / n) as f32 * 360.0 / reps as f32;
                (
                    rotate_degrees(points[i % n].0, angle, center),
                    points[i % n].1,
                )
            })
            .collect()
    }
}

fn rotate_degrees(p: [f32; 2], degrees: f32, center: [f32; 2]) -> [f32; 2] {
    let a = degrees / 360.0 * 2.0 * PI;
    let off = sub(p, center);
    [
        off[0] * a.cos() - off[1] * a.sin() + center[0],
        off[0] * a.sin() + off[1] * a.cos() + center[1],
    ]
}

fn angle_degrees(p: [f32; 2]) -> f32 {
    p[1].atan2(p[0]) * 180.0 / PI
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn length(a: [f32; 2]) -> f32 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}
