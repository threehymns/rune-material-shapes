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
// Ported from the vector helpers in androidx.graphics.shapes.Point, modified here; see README.md.

//! Small 2D vector helpers shared by the rounded-polygon geometry.

pub(crate) fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

pub(crate) fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub(crate) fn mul(a: [f32; 2], k: f32) -> [f32; 2] {
    [a[0] * k, a[1] * k]
}

pub(crate) fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

pub(crate) fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

pub(crate) fn length(a: [f32; 2]) -> f32 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}

pub(crate) fn rotate90(a: [f32; 2]) -> [f32; 2] {
    [-a[1], a[0]]
}

pub(crate) fn direction(a: [f32; 2]) -> Option<[f32; 2]> {
    let d = length(a);
    (d > 0.0).then(|| [a[0] / d, a[1] / d])
}

pub(crate) fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

pub(crate) fn average(points: &[[f32; 2]]) -> [f32; 2] {
    let mut sum = [0.0, 0.0];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
    }
    [sum[0] / points.len() as f32, sum[1] / points.len() as f32]
}

/// Android's `convex`: whether the turn from `prev` through `curr` to `next` is clockwise.
pub(crate) fn is_convex(prev: [f32; 2], curr: [f32; 2], next: [f32; 2]) -> bool {
    let into = sub(curr, prev);
    let out = sub(next, curr);
    into[0] * out[1] - into[1] * out[0] > 0.0
}
