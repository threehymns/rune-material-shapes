//! A Rune [`Playground`] of the whole Material shape library.
//!
//! `App::playground(rune_material_shapes::playground())` adds a "Material 3" group with an
//! all-shapes story, one per category, an animated Morph story and a Surfaces story. The shape
//! stories start on the Filled variant; 2 and 3 in the side column show the outlines and the soft
//! two-tone paint. `cargo run --example material-shapes -- --playground "Material 3/All shapes"`
//! opens the example at this playground.

use rune::{Playground, Stage, Story};

use crate::Category;
use crate::paint::PaintStyle;
use crate::surfaces::{MorphSurfaces, morph_surfaces};
use crate::view::{Gallery, MorphDemo, gallery, morph_demo};

const CATEGORIES: [Category; 5] = [
    Category::Basic,
    Category::Cookies,
    Category::Clovers,
    Category::Bursts,
    Category::Unique,
];

/// The shape library as playground stories, for an app with any state type.
pub fn playground<S: 'static>() -> Playground<S> {
    let mut playground = Playground::new().story(
        Story::new("Material 3/All shapes", Gallery::new, gallery)
            .stage(Stage::Scroll)
            .variant("Filled", |g: &mut Gallery| g.style = PaintStyle::Filled)
            .variant("Outlined", |g: &mut Gallery| g.style = PaintStyle::Outlined)
            .variant("Soft", |g: &mut Gallery| g.style = PaintStyle::Soft),
    );
    for category in CATEGORIES {
        playground = playground.story(
            Story::new(
                story_name(category),
                move || Gallery::in_category(category),
                gallery,
            )
            .stage(Stage::Scroll)
            .variant("Filled", |g: &mut Gallery| g.style = PaintStyle::Filled)
            .variant("Outlined", |g: &mut Gallery| g.style = PaintStyle::Outlined)
            .variant("Soft", |g: &mut Gallery| g.style = PaintStyle::Soft),
        );
    }
    playground
        .story(
            Story::new("Material 3/Morph", MorphDemo::new, morph_demo)
                .stage(Stage::Scroll)
                .poll(|demo: &mut MorphDemo, now, _| demo.tick(now))
                .variant("Filled", |d: &mut MorphDemo| {
                    d.set_style(PaintStyle::Filled)
                })
                .variant("Outlined", |d: &mut MorphDemo| {
                    d.set_style(PaintStyle::Outlined)
                })
                .variant("Soft", |d: &mut MorphDemo| d.set_style(PaintStyle::Soft)),
        )
        .story(
            Story::new("Material 3/Surfaces", MorphSurfaces::new, morph_surfaces)
                .stage(Stage::Scroll)
                .poll(|s: &mut MorphSurfaces, now, _| s.tick(now)),
        )
}

fn story_name(category: Category) -> &'static str {
    match category {
        Category::Basic => "Material 3/Basic",
        Category::Cookies => "Material 3/Cookies",
        Category::Clovers => "Material 3/Clovers",
        Category::Bursts => "Material 3/Bursts",
        Category::Unique => "Material 3/Unique",
    }
}
