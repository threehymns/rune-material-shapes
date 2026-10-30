//! Material 3 Expressive's shape library for Rune.
//!
//! Android's Material 3 Expressive update added a library of 35 shapes — circles and pills,
//! cookies, clovers, bursts and one-offs such as the heart and the bun — modeled as
//! [`RoundedPolygon`]s that can be painted, scaled, and used as the outline of an image or an
//! avatar. This crate ports that geometry to Rust and paints it with Rune's vector [`Shape`]s.
//!
//! [`MaterialShape`] is the catalog: `MaterialShape::Cookie12Sided.polygon()` is Android's
//! `MaterialShapes.Cookie12Sided`, normalized into the unit square. [`paint`] turns a polygon
//! into a Rune [`Shape`], and [`shape_canvas`] into a square [`El`]:
//!
//! ```ignore
//! use rune_material_shapes::{MaterialShape, PaintStyle, shape_canvas};
//! use rune::tok;
//!
//! let leaf = shape_canvas(
//!     MaterialShape::Clover4Leaf,
//!     96.0,
//!     PaintStyle::Filled,
//!     tok::accent(4, true),
//! );
//! ```
//!
//! The geometry is a direct port of `androidx.graphics.shapes.RoundedPolygon` (Apache-2.0),
//! including its corner cut adjustment, circular-arc approximation, smoothing flanks and
//! approximate bounds, and the 35 definitions are transcribed from
//! `androidx.compose.material3.MaterialShapes` (Apache-2.0). Shapes are normalized uniformly,
//! so an oval stays an oval rather than stretching into a circle.
//!
//! [`Morph`] animates any shape into any other, using Android's feature mapping and curve cutting:
//!
//! ```ignore
//! use rune_material_shapes::{MaterialShape, Morph};
//!
//! let morph = Morph::new(MaterialShape::Circle.polygon(), MaterialShape::Heart.polygon());
//! let halfway = morph.polygon(0.5);
//! ```
//!
//! [`morph_paint`] and [`shaped`] put that shape beneath any Rune element's content, so buttons,
//! cards and avatars can change silhouette without a separate canvas.
//!
//! # The playground
//!
//! With the default `rune` feature, [`playground`] is a Rune [`Playground`] with an all-shapes
//! story, one per category, a Morph story driven by [`MorphDemo`], and a Surfaces story where a
//! card, button and avatar morph under their content. The example app wires it up:
//!
//! ```sh
//! cargo run --example material-shapes
//! cargo run --example material-shapes -- --playground "Material 3/All shapes"
//! cargo run --example material-shapes -- --playground "Material 3/Morph"
//! cargo run --example material-shapes -- --playground "Material 3/Surfaces"
//! ```
//!
//! [`Shape`]: rune::Shape
//! [`El`]: rune::El
//! [`Playground`]: rune::Playground

mod catalog;
mod geometry;
mod math;
mod morph;

pub use catalog::{Category, MaterialShape, MaterialShapes};
pub use geometry::{CornerRounding, Cubic, DISTANCE_EPSILON, Feature, FeatureKind, RoundedPolygon};
pub use morph::Morph;

#[cfg(feature = "rune")]
mod paint;
#[cfg(feature = "rune")]
mod playground;
#[cfg(feature = "rune")]
mod surfaces;
#[cfg(feature = "rune")]
mod view;

#[cfg(feature = "rune")]
pub use paint::{PAINT_TOLERANCE, PaintStyle, morph_paint, paint, shape_canvas, shaped};
#[cfg(feature = "rune")]
pub use playground::playground;
#[cfg(feature = "rune")]
pub use surfaces::{MorphSurfaces, morph_surfaces};
#[cfg(feature = "rune")]
pub use view::{Gallery, MorphDemo, gallery, morph_demo};

#[cfg(all(test, feature = "rune"))]
mod playground_tests;
#[cfg(test)]
mod tests;
