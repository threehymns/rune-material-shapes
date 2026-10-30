//! The gallery view: a Material 3 Expressive shape browser, used by the example app and the
//! playground stories.
//!
//! It is a normal Rune view over [`Gallery`], so an app can embed it, and the playground can show
//! it live. Clicking a tile selects it and the large preview above the grid follows.

use rune::tok::{self, type_};
use rune::{Align, Color, Cx, Drag, El, Phase, TextStyle, canvas, col, el, row, text};

use crate::paint::{PaintStyle, paint};
use crate::{Category, MaterialShape, Morph, RoundedPolygon};

/// The gallery's state: which shape is previewed, which category is shown, and how shapes are
/// painted.
#[derive(Clone, Debug, PartialEq)]
pub struct Gallery {
    pub selected: MaterialShape,
    /// `Some` shows one category; `None` shows all 35.
    pub category: Option<Category>,
    /// How every tile and the preview are painted.
    pub style: PaintStyle,
}

impl Default for Gallery {
    fn default() -> Self {
        Gallery::new()
    }
}

impl Gallery {
    /// All 35 shapes, with the circle previewed.
    pub fn new() -> Gallery {
        Gallery {
            selected: MaterialShape::Circle,
            category: None,
            style: PaintStyle::Filled,
        }
    }

    /// One category, with its first shape previewed.
    pub fn in_category(category: Category) -> Gallery {
        let selected = MaterialShape::in_category(category)
            .next()
            .unwrap_or(MaterialShape::Circle);
        Gallery {
            selected,
            category: Some(category),
            style: PaintStyle::Filled,
        }
    }

    /// The shapes this gallery lists.
    pub fn shapes(&self) -> Vec<MaterialShape> {
        match self.category {
            Some(category) => MaterialShape::in_category(category).collect(),
            None => MaterialShape::ALL.to_vec(),
        }
    }
}

/// The Material 3 Expressive shape gallery.
///
/// In a playground story, use [`crate::playground`] and this as the story's view. In an app, wrap
/// it in a scroll view when the window is shorter than 35 tiles.
pub fn gallery(s: &Gallery, cx: &mut Cx<Gallery>) -> El {
    let dark = cx.system_dark();
    let t = tok::theme(dark);
    let shapes = s.shapes();
    let selected_color = shape_color(s.selected, dark);
    let style = s.style;

    let mut tiles = Vec::with_capacity(shapes.len());
    for shape in &shapes {
        let color = shape_color(*shape, dark);
        let selected = *shape == s.selected;
        tiles.push(shape_tile(*shape, color, selected, style, &t));
    }

    let section = s.category.map_or("All shapes", Category::label);
    let subtitle = match s.category {
        Some(category) => format!(
            "{} of Android's 35 Material Shapes, drawn with Rune vectors.",
            category.label()
        ),
        None => "Android's 35-shape Material Shapes library, drawn with Rune vectors.".to_owned(),
    };

    col()
        .key(format!(
            "material-shapes-{}",
            s.category.map_or("all", Category::label)
        ))
        .w_full()
        .padding(28.0)
        .gap(22.0)
        .fill(t.bg)
        .child(
            col()
                .gap(6.0)
                .child(
                    text("Material 3 Expressive shapes")
                        .font(TextStyle::new(29.0, 650.0))
                        .color(t.text),
                )
                .child(text(subtitle).font(type_::BODY).color(t.text_secondary)),
        )
        .child(hero(s.selected, selected_color, style, dark))
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text(section).font(type_::SECTION).color(t.text_secondary))
                .child(
                    text(format!("{} shapes", shapes.len()))
                        .font(type_::LABEL)
                        .color(t.text_secondary),
                ),
        )
        .child(row().flex_wrap().w_full().gaps(14.0, 16.0).children(tiles))
}

fn hero(selected: MaterialShape, color: Color, style: PaintStyle, dark: bool) -> El {
    let t = tok::theme(dark);
    row()
        .key("material-shape-hero")
        .w_full()
        .gap(24.0)
        .padding(20.0)
        .radius(32.0)
        .fill(t.card)
        .border(1.0, t.card_border)
        .align(Align::Center)
        .child(canvas(paint(&selected.polygon(), style, color)).size(136.0, 136.0))
        .child(
            col()
                .gap(5.0)
                .child(
                    text(selected.label())
                        .font(TextStyle::new(25.0, 650.0))
                        .color(t.text),
                )
                .child(
                    text(format!(
                        "{}  ·  {}",
                        selected.category().label(),
                        selected.name()
                    ))
                    .font(type_::LABEL)
                    .color(t.text_secondary),
                )
                .child(
                    text("Click a shape below to preview it.")
                        .font(type_::BODY)
                        .color(t.text_secondary),
                ),
        )
}

fn shape_tile(
    shape: MaterialShape,
    color: Color,
    selected: bool,
    style: PaintStyle,
    t: &tok::Theme,
) -> El {
    let background = if selected { color.alpha(0.16) } else { t.card };
    let border = if selected {
        color.alpha(0.55)
    } else {
        t.card_border
    };
    col()
        .key(format!("material-shape-{}", shape.name()))
        .w(140.0)
        .padding(12.0)
        .gap(8.0)
        .center()
        .radius(24.0)
        .fill(background)
        .border(1.0, border)
        .child(canvas(paint(&shape.polygon(), style, color)).size(80.0, 80.0))
        .child(text(shape.label()).font(type_::LABEL).color(t.text))
        .on_press(move |s: &mut Gallery| s.selected = shape)
}

fn shape_color(shape: MaterialShape, dark: bool) -> Color {
    let index = MaterialShape::ALL
        .iter()
        .position(|s| *s == shape)
        .unwrap_or(0);
    tok::accent(index, dark)
}

/// A live morph between two shapes, for the playground's Morph story and for an app that wants
/// its own control panel.
///
/// [`MorphDemo::tick`] advances the animation; the playground calls it through `Story::poll`, so
/// the preview is one repaint per frame and no timers. The motion ping-pongs between the two
/// endpoints, easing in and out, until it is paused.
pub struct MorphDemo {
    from: MaterialShape,
    to: MaterialShape,
    progress: f32,
    /// `1.0` towards `to`, `-1.0` back towards `from`.
    direction: f32,
    playing: bool,
    style: PaintStyle,
    duration: f32,
    morph: Morph,
    /// The clock at the last tick, so a paused story or a changed endpoint doesn't jump.
    last: Option<f64>,
    random_state: u32,
}

impl Default for MorphDemo {
    fn default() -> Self {
        MorphDemo::new()
    }
}

impl MorphDemo {
    /// A circle morphing into a heart.
    pub fn new() -> MorphDemo {
        MorphDemo::between(MaterialShape::Circle, MaterialShape::Heart)
    }

    /// A paused-ready morph between `from` and `to`, starting at `from`.
    pub fn between(from: MaterialShape, to: MaterialShape) -> MorphDemo {
        MorphDemo {
            from,
            to,
            progress: 0.0,
            direction: 1.0,
            playing: true,
            style: PaintStyle::Filled,
            duration: 1.6,
            morph: Morph::new(from.polygon(), to.polygon()),
            last: None,
            random_state: 0x9e37_79b9,
        }
    }

    pub fn from(&self) -> MaterialShape {
        self.from
    }

    pub fn to(&self) -> MaterialShape {
        self.to
    }

    /// Linear time along the animation, before easing.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// The eased progress the preview paints.
    pub fn eased_progress(&self) -> f32 {
        let t = self.progress;
        t * t * (3.0 - 2.0 * t)
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn set_duration(&mut self, seconds: f32) {
        self.duration = seconds.max(0.05);
    }

    pub fn style(&self) -> PaintStyle {
        self.style
    }

    pub fn set_style(&mut self, style: PaintStyle) {
        self.style = style;
    }

    pub fn morph(&self) -> &Morph {
        &self.morph
    }

    /// The shape at this frame's eased progress.
    pub fn polygon(&self) -> RoundedPolygon {
        self.morph.polygon(self.eased_progress())
    }

    /// Morph from `shape`, restarting the animation.
    pub fn set_from(&mut self, shape: MaterialShape) {
        self.from = shape;
        self.rebuild();
    }

    /// Morph into `shape`, restarting the animation.
    pub fn set_to(&mut self, shape: MaterialShape) {
        self.to = shape;
        self.rebuild();
    }

    /// Reverse the direction of the morph.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.rebuild();
    }

    /// Pick two different shapes and morph between them.
    pub fn randomize(&mut self) {
        let a = self.next_random();
        let mut b = self.next_random();
        if a == b {
            b = (b + 1) % MaterialShape::ALL.len();
        }
        self.from = MaterialShape::ALL[a];
        self.to = MaterialShape::ALL[b];
        self.rebuild();
    }

    pub fn toggle(&mut self) {
        if self.playing {
            self.playing = false;
        } else {
            if self.progress >= 1.0 {
                self.direction = -1.0;
            } else if self.progress <= 0.0 {
                self.direction = 1.0;
            }
            self.playing = true;
            self.last = None;
        }
    }

    /// Play again from `from`.
    pub fn restart(&mut self) {
        self.progress = 0.0;
        self.direction = 1.0;
        self.playing = true;
        self.last = None;
    }

    /// Scrub to `progress`, pausing there.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.playing = false;
        self.last = None;
        if self.progress >= 1.0 {
            self.direction = -1.0;
        } else if self.progress <= 0.0 {
            self.direction = 1.0;
        }
    }

    /// Advance the animation to `now` (the runtime's clock, in seconds). Returns whether the
    /// preview changed and a frame is needed.
    pub fn tick(&mut self, now: f64) -> bool {
        let Some(last) = self.last.replace(now) else {
            return false;
        };
        if !self.playing {
            return false;
        }
        // A suspended or restored window can hand over a large gap; never fast-forward the
        // shape in one step.
        let dt = (now - last).clamp(0.0, 0.1) as f32;
        if dt == 0.0 {
            return false;
        }
        let before = self.progress;
        self.progress += self.direction * dt / self.duration;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.direction = -1.0;
        } else if self.progress <= 0.0 {
            self.progress = 0.0;
            self.direction = 1.0;
        }
        self.progress != before
    }

    fn rebuild(&mut self) {
        self.morph = Morph::new(self.from.polygon(), self.to.polygon());
        self.progress = 0.0;
        self.direction = 1.0;
        self.playing = true;
        self.last = None;
    }

    fn next_random(&mut self) -> usize {
        self.random_state = self
            .random_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.random_state >> 16) as usize % MaterialShape::ALL.len()
    }
}

/// The Morph playground story: a ping-ponging morph, a play/pause button, a scrubber, and a row
/// of all 35 shapes for each endpoint.
pub fn morph_demo(s: &MorphDemo, cx: &mut Cx<MorphDemo>) -> El {
    let dark = cx.system_dark();
    let t = tok::theme(dark);
    let accent = shape_color(s.to, dark);
    let progress = s.eased_progress();
    let state = if s.playing { "Playing" } else { "Paused" };
    let title = format!("{} → {}", s.from.label(), s.to.label());
    let subtitle = format!(
        "{state} · {}% · {} curves matched",
        (progress * 100.0).round() as i32,
        s.morph.curve_count()
    );
    let preview = paint(&s.polygon(), s.style, accent);

    col()
        .key("material-morph")
        .w_full()
        .padding(28.0)
        .gap(20.0)
        .fill(t.bg)
        .child(
            col()
                .gap(6.0)
                .child(
                    text("Material shape morph")
                        .font(TextStyle::new(29.0, 650.0))
                        .color(t.text),
                )
                .child(
                    text("Any of Android's 35 shapes, animated into any other with matched corners and cut curves.")
                        .font(type_::BODY)
                        .color(t.text_secondary),
                ),
        )
        .child(
            row()
                .w_full()
                .gap(20.0)
                .padding(20.0)
                .radius(32.0)
                .fill(t.card)
                .border(1.0, t.card_border)
                .align(Align::Center)
                .child(canvas(preview).key("material-morph-preview").size(200.0, 200.0))
                .child(
                    col()
                        .w(300.0)
                        .gap(10.0)
                        .child(
                            text(title)
                                .key("material-morph-title")
                                .font(TextStyle::new(26.0, 650.0))
                                .color(t.text)
                                .wrap(),
                        )
                        .child(text(subtitle).key("material-morph-state").font(type_::LABEL).color(t.text_secondary))
                        .child(
                            row()
                                .gap(8.0)
                                .child(action_button(
                                    "material-morph-play",
                                    if s.playing { "Pause" } else { "Play" },
                                    t,
                                    accent,
                                    true,
                                    |s: &mut MorphDemo| s.toggle(),
                                ))
                                .child(action_button(
                                    "material-morph-swap",
                                    "Swap",
                                    t,
                                    accent,
                                    false,
                                    |s: &mut MorphDemo| s.swap(),
                                ))
                                .child(action_button(
                                    "material-morph-random",
                                    "Random",
                                    t,
                                    accent,
                                    false,
                                    |s: &mut MorphDemo| s.randomize(),
                                )),
                        )
                        .child(morph_slider(s, t, accent)),
                ),
        )
        .child(endpoint_picker(
            "Morph from",
            s.from,
            "material-morph-from",
            t,
            dark,
            MorphDemo::set_from,
        ))
        .child(endpoint_picker(
            "Morph to",
            s.to,
            "material-morph-to",
            t,
            dark,
            MorphDemo::set_to,
        ))
}

fn action_button(
    key: &'static str,
    label: &'static str,
    t: tok::Theme,
    accent: Color,
    primary: bool,
    run: impl Fn(&mut MorphDemo) + 'static,
) -> El {
    row()
        .key(key)
        .role("button")
        .label(label)
        .h(32.0)
        .px(14.0)
        .center()
        .radius(16.0)
        .fill(if primary { accent } else { t.bg })
        .border(1.0, if primary { accent } else { t.card_border })
        .hover(t.hover)
        .press(t.press)
        .on_press(run)
        .child(
            text(label)
                .font(type_::LABEL)
                .color(if primary { Color::WHITE } else { t.text }),
        )
}

fn morph_slider(s: &MorphDemo, t: tok::Theme, accent: Color) -> El {
    const WIDTH: f32 = 300.0;
    const HEIGHT: f32 = 20.0;
    const KNOB: f32 = 14.0;
    const BAR: f32 = 4.0;
    let travel = WIDTH - KNOB;
    let u = s.progress();
    el().key("material-morph-progress")
        .role("slider")
        .label("Morph progress")
        .size(WIDTH, HEIGHT)
        .on_drag(move |s: &mut MorphDemo, d: &mut Drag, tx| {
            if matches!(d.phase, Phase::Down | Phase::Move) {
                s.set_progress((d.x - KNOB / 2.0) / travel);
                tx.instant();
            }
        })
        .child(
            el().key("track")
                .size(WIDTH, BAR)
                .absolute(0.0, (HEIGHT - BAR) / 2.0)
                .radius(BAR / 2.0)
                .fill(t.control_track),
        )
        .child(
            el().key("fill")
                .size(KNOB / 2.0 + u * travel, BAR)
                .absolute(0.0, (HEIGHT - BAR) / 2.0)
                .radius(BAR / 2.0)
                .fill(accent),
        )
        .child(
            el().key("knob")
                .size(KNOB, KNOB)
                .absolute(u * travel, (HEIGHT - KNOB) / 2.0)
                .radius(KNOB / 2.0)
                .fill(t.knob)
                .shadow(1.0, 3.0, t.shadow),
        )
}

fn endpoint_picker(
    title: &'static str,
    selected: MaterialShape,
    prefix: &'static str,
    t: tok::Theme,
    dark: bool,
    choose: fn(&mut MorphDemo, MaterialShape),
) -> El {
    let mut chips = Vec::with_capacity(MaterialShape::ALL.len());
    for shape in MaterialShape::ALL {
        let color = shape_color(shape, dark);
        let is_selected = shape == selected;
        chips.push(
            el().key(format!("{prefix}-{}", shape.name()))
                .role("button")
                .label(shape.label())
                .size(42.0, 42.0)
                .center()
                .radius(14.0)
                .fill(if is_selected {
                    color.alpha(0.18)
                } else {
                    t.card
                })
                .border(
                    1.0,
                    if is_selected {
                        color.alpha(0.65)
                    } else {
                        t.card_border
                    },
                )
                .hover(t.hover)
                .on_press(move |s: &mut MorphDemo| choose(s, shape))
                .child(canvas(paint(&shape.polygon(), PaintStyle::Filled, color)).size(26.0, 26.0)),
        );
    }
    col()
        .gap(8.0)
        .w_full()
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text(title).font(type_::SECTION).color(t.text_secondary))
                .child(
                    text(selected.label())
                        .font(type_::LABEL)
                        .color(t.text_secondary),
                ),
        )
        .child(row().flex_wrap().w_full().gaps(8.0, 8.0).children(chips))
}
