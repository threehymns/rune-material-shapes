//! The playground wiring: every story opens, lists its shapes, paints its variants, and a tile
//! click selects.

use crate::paint::PaintStyle;
use crate::{Gallery, MaterialShape, MorphDemo, MorphSurfaces, gallery, playground};
use rune::{App, AppSpec, Cx, El, Env, Runtime};

fn view(g: &Gallery, cx: &mut Cx<Gallery>) -> El {
    gallery(g, cx)
}

fn runtime_at(size: (f32, f32)) -> Runtime<Gallery> {
    let spec = AppSpec::new("material shapes test", size, Gallery::new, view);
    App::new(spec)
        .playground(playground())
        .headless_runtime(size, Env::new(false, 1.0))
}

fn runtime() -> Runtime<Gallery> {
    runtime_at((1100.0, 860.0))
}

fn tiles(rt: &Runtime<Gallery>) -> usize {
    rt.elements()
        .iter()
        .filter(|e| {
            e.key
                .as_deref()
                .is_some_and(|k| k.starts_with("material-shape-") && k != "material-shape-hero")
        })
        .count()
}

#[test]
fn the_all_shapes_story_has_a_tile_for_every_shape() {
    let mut rt = runtime();
    rt.open_playground("Material 3/All shapes", None);
    assert_eq!(
        rt.playground_state::<Gallery>().map(|g| g.category),
        Some(None),
        "the all-shapes story lists every category"
    );
    assert_eq!(tiles(&rt), 35);
    assert!(rt.find("material-shape-hero").is_some());
}

#[test]
fn each_category_story_lists_just_its_shapes() {
    for (name, category) in [
        ("Material 3/Basic", crate::Category::Basic),
        ("Material 3/Cookies", crate::Category::Cookies),
        ("Material 3/Clovers", crate::Category::Clovers),
        ("Material 3/Bursts", crate::Category::Bursts),
        ("Material 3/Unique", crate::Category::Unique),
    ] {
        let mut rt = runtime();
        rt.open_playground(name, None);
        let state = rt.playground_state::<Gallery>().expect("the story's state");
        assert_eq!(state.category, Some(category));
        assert_eq!(
            state.selected,
            MaterialShape::in_category(category).next().unwrap()
        );
        assert_eq!(
            tiles(&rt),
            MaterialShape::in_category(category).count(),
            "{name}"
        );
    }
}

#[test]
fn the_paint_variants_switch_every_tile() {
    for (name, style) in [
        ("Outlined", PaintStyle::Outlined),
        ("Soft", PaintStyle::Soft),
        ("Filled", PaintStyle::Filled),
    ] {
        let mut rt = runtime();
        rt.open_playground("Material 3/All shapes", Some(name));
        assert_eq!(
            rt.playground_state::<Gallery>().map(|g| g.style),
            Some(style)
        );
    }
}

#[test]
fn clicking_a_tile_previews_that_shape() {
    let mut rt = runtime();
    rt.open_playground("Material 3/All shapes", None);
    let (x, y) = rt.find("material-shape-Square").expect("a square tile");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    assert_eq!(
        rt.playground_state::<Gallery>().map(|g| g.selected),
        Some(MaterialShape::Square)
    );
}

#[test]
fn the_morph_story_offers_both_endpoints_for_every_shape() {
    let mut rt = runtime();
    rt.open_playground("Material 3/Morph", None);
    let demo = rt.playground_state::<MorphDemo>().expect("the morph state");
    assert_eq!(demo.from(), MaterialShape::Circle);
    assert_eq!(demo.to(), MaterialShape::Heart);
    for prefix in ["material-morph-from-", "material-morph-to-"] {
        for shape in MaterialShape::ALL {
            assert!(
                rt.find(&format!("{prefix}{}", shape.name())).is_some(),
                "{prefix}{}",
                shape.name()
            );
        }
    }
    assert!(rt.find("material-morph-preview").is_some());
    assert_eq!(
        rt.elements()
            .iter()
            .filter(|e| e.key.as_deref() == Some("material-morph-progress"))
            .count(),
        1
    );
}

#[test]
fn choosing_either_endpoint_restarts_the_morph() {
    let mut rt = runtime_at((1100.0, 1600.0));
    rt.open_playground("Material 3/Morph", None);
    for _ in 0..20 {
        rt.frame(1.0 / 60.0);
    }
    assert!(rt.playground_state::<MorphDemo>().unwrap().progress() > 0.0);

    let (x, y) = rt
        .find("material-morph-from-Square")
        .expect("a square source");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let demo = rt.playground_state::<MorphDemo>().unwrap();
    assert_eq!(demo.from(), MaterialShape::Square);
    assert_eq!(demo.to(), MaterialShape::Heart);
    assert_eq!(demo.progress(), 0.0);
    assert!(demo.playing());

    let (x, y) = rt.find("material-morph-to-Bun").expect("a bun target");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let demo = rt.playground_state::<MorphDemo>().unwrap();
    assert_eq!(demo.from(), MaterialShape::Square);
    assert_eq!(demo.to(), MaterialShape::Bun);
    assert_eq!(demo.progress(), 0.0);
}

#[test]
fn the_morph_story_animates_and_pauses() {
    let mut rt = runtime();
    rt.open_playground("Material 3/Morph", None);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
    let moving = rt.playground_state::<MorphDemo>().unwrap().progress();
    assert!(moving > 0.2, "the preview advanced: {moving}");

    let (x, y) = rt.find("material-morph-play").expect("the play button");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let paused = rt.playground_state::<MorphDemo>().unwrap().progress();
    assert!(!rt.playground_state::<MorphDemo>().unwrap().playing());
    for _ in 0..15 {
        rt.frame(1.0 / 60.0);
    }
    let still = rt.playground_state::<MorphDemo>().unwrap().progress();
    assert_eq!(paused, still, "a paused morph stays put");
}

#[test]
fn a_morph_ping_pongs_between_its_endpoints() {
    let mut demo = MorphDemo::between(MaterialShape::Circle, MaterialShape::Heart);
    demo.set_duration(1.0);
    assert!(!demo.tick(0.0), "the first tick only starts the clock");
    let mut now = 0.0;
    for _ in 0..12 {
        now += 0.1;
        demo.tick(now);
    }
    assert!(
        demo.progress() <= 1.0 && demo.progress() > 0.0,
        "{}",
        demo.progress()
    );
    assert!(demo.progress() < 0.95, "it has turned around at the end");
    for _ in 0..20 {
        now += 0.1;
        demo.tick(now);
    }
    assert!(
        demo.progress() > 0.0 && demo.progress() < 1.0,
        "it comes back: {}",
        demo.progress()
    );

    demo.set_progress(1.0);
    assert!(!demo.playing());
    assert_eq!(demo.progress(), 1.0);
    let end = MaterialShape::Heart.polygon();
    for (a, b) in demo.polygon().bounds().iter().zip(end.bounds()) {
        assert!((a - b).abs() < 5e-3, "the end is the heart: {a} vs {b}");
    }
}

#[test]
fn the_surfaces_story_has_a_card_button_and_avatar() {
    let mut rt = runtime_at((1100.0, 1000.0));
    rt.open_playground("Material 3/Surfaces", None);
    let surfaces = rt
        .playground_state::<MorphSurfaces>()
        .expect("the surfaces state");
    assert_eq!(surfaces.card().from(), MaterialShape::Square);
    assert_eq!(surfaces.button().from(), MaterialShape::Pill);
    assert_eq!(surfaces.avatar().from(), MaterialShape::Circle);
    for key in [
        "morph-surface-card",
        "morph-surface-button",
        "morph-surface-avatar",
    ] {
        assert!(rt.find(key).is_some(), "{key}");
    }
    assert!(rt.find("morph-surfaces-play").is_some());
    assert!(rt.find("morph-surfaces-next").is_some());
}

#[test]
fn surfaces_all_animate_and_pause_together() {
    let mut rt = runtime_at((1100.0, 1000.0));
    rt.open_playground("Material 3/Surfaces", None);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
    let surfaces = rt.playground_state::<MorphSurfaces>().unwrap();
    assert!(surfaces.card().progress() > 0.0);
    assert!(surfaces.button().progress() > 0.0);
    assert!(surfaces.avatar().progress() > 0.0);

    let (x, y) = rt.find("morph-surfaces-play").expect("the pause button");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let surfaces = rt.playground_state::<MorphSurfaces>().unwrap();
    assert!(!surfaces.playing());
    let (card, button, avatar) = (
        surfaces.card().progress(),
        surfaces.button().progress(),
        surfaces.avatar().progress(),
    );
    for _ in 0..15 {
        rt.frame(1.0 / 60.0);
    }
    let surfaces = rt.playground_state::<MorphSurfaces>().unwrap();
    assert_eq!(surfaces.card().progress(), card);
    assert_eq!(surfaces.button().progress(), button);
    assert_eq!(surfaces.avatar().progress(), avatar);
}

#[test]
fn next_shapes_retargets_every_surface_and_the_button_cycles_its_own() {
    let mut rt = runtime_at((1100.0, 1000.0));
    rt.open_playground("Material 3/Surfaces", None);
    let (card, button, avatar) = {
        let s = rt.playground_state::<MorphSurfaces>().unwrap();
        (s.card().to(), s.button().to(), s.avatar().to())
    };
    let (x, y) = rt
        .find("morph-surfaces-next")
        .expect("the next shapes button");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let s = rt.playground_state::<MorphSurfaces>().unwrap();
    assert_ne!(s.card().to(), card);
    assert_ne!(s.button().to(), button);
    assert_ne!(s.avatar().to(), avatar);
    assert_eq!(s.card().progress(), 0.0);
    assert!(s.card().playing());

    let before = s.button().to();
    let (x, y) = rt
        .find("morph-surface-button")
        .expect("the morphing button");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let s = rt.playground_state::<MorphSurfaces>().unwrap();
    assert_ne!(s.button().to(), before);
    assert_eq!(s.button().progress(), 0.0);
}

#[test]
fn the_morph_variants_change_the_paint_style() {
    for (name, style) in [
        ("Outlined", PaintStyle::Outlined),
        ("Soft", PaintStyle::Soft),
        ("Filled", PaintStyle::Filled),
    ] {
        let mut rt = runtime();
        rt.open_playground("Material 3/Morph", Some(name));
        assert_eq!(
            rt.playground_state::<MorphDemo>().map(|d| d.style()),
            Some(style)
        );
    }
}
