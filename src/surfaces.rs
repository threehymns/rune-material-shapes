//! Shape-shifting Rune surfaces: a card, a button and an avatar whose backgrounds morph between
//! Material shapes.
//!
//! Each surface is an ordinary Rune element with the morph painted beneath its content by
//! [`rune::El::shape`]. Rune has no arbitrary path clipping, so the content is not clipped to the
//! silhouette; the shapes used here stay convex enough for inset content to remain inside, and the
//! element's hit area stays rectangular.

use rune::tok::{self, type_};
use rune::{Align, Color, Cx, El, TextStyle, col, el, row, text};

use crate::MaterialShape;
use crate::paint::{PaintStyle, morph_paint};
use crate::view::MorphDemo;

/// Shapes a card cycles through when it asks for the next one.
const CARD_TARGETS: [MaterialShape; 4] = [
    MaterialShape::Cookie6Sided,
    MaterialShape::Arch,
    MaterialShape::Puffy,
    MaterialShape::Gem,
];
/// Shapes a button cycles through. All are convex enough to keep a centred label inside.
const BUTTON_TARGETS: [MaterialShape; 4] = [
    MaterialShape::Cookie4Sided,
    MaterialShape::Arch,
    MaterialShape::Gem,
    MaterialShape::Cookie12Sided,
];
/// Shapes an avatar cycles through.
const AVATAR_TARGETS: [MaterialShape; 4] = [
    MaterialShape::Heart,
    MaterialShape::Clover4Leaf,
    MaterialShape::Flower,
    MaterialShape::Circle,
];

/// Three independently morphing surfaces, for the playground's Surfaces story and for an app's
/// own settings or profile chrome.
///
/// The animation is driven by [`MorphSurfaces::tick`], called from `Story::poll`; each surface
/// ping-pongs on its own duration so they drift out of phase.
pub struct MorphSurfaces {
    card: MorphDemo,
    button: MorphDemo,
    avatar: MorphDemo,
}

impl Default for MorphSurfaces {
    fn default() -> Self {
        MorphSurfaces::new()
    }
}

impl MorphSurfaces {
    /// A square card, a pill button and a circular avatar, each morphing into a different shape.
    pub fn new() -> MorphSurfaces {
        let mut card = MorphDemo::between(MaterialShape::Square, MaterialShape::Cookie6Sided);
        card.set_duration(2.4);
        let mut button = MorphDemo::between(MaterialShape::Pill, MaterialShape::Cookie4Sided);
        button.set_duration(1.5);
        let mut avatar = MorphDemo::between(MaterialShape::Circle, MaterialShape::Heart);
        avatar.set_duration(2.0);
        MorphSurfaces {
            card,
            button,
            avatar,
        }
    }

    /// The card's animation, including its current shape pair and progress.
    pub fn card(&self) -> &MorphDemo {
        &self.card
    }

    /// The button's animation.
    pub fn button(&self) -> &MorphDemo {
        &self.button
    }

    /// The avatar's animation.
    pub fn avatar(&self) -> &MorphDemo {
        &self.avatar
    }

    /// Whether any surface is still moving.
    pub fn playing(&self) -> bool {
        self.card.playing() || self.button.playing() || self.avatar.playing()
    }

    /// Pause every surface, or resume every paused one.
    pub fn toggle_all(&mut self) {
        let pause = self.playing();
        for demo in [&mut self.card, &mut self.button, &mut self.avatar] {
            if demo.playing() == pause {
                demo.toggle();
            }
        }
    }

    /// Advance every surface. True when at least one frame of animation is needed.
    pub fn tick(&mut self, now: f64) -> bool {
        let card = self.card.tick(now);
        let button = self.button.tick(now);
        let avatar = self.avatar.tick(now);
        card || button || avatar
    }

    /// Morph the card into its next shape, restarting from its source shape.
    pub fn next_card(&mut self) {
        next_target(&mut self.card, &CARD_TARGETS);
    }

    /// Morph the button into its next shape.
    pub fn next_button(&mut self) {
        next_target(&mut self.button, &BUTTON_TARGETS);
    }

    /// Morph the avatar into its next shape.
    pub fn next_avatar(&mut self) {
        next_target(&mut self.avatar, &AVATAR_TARGETS);
    }

    /// Send every surface to its next shape.
    pub fn next_shapes(&mut self) {
        self.next_card();
        self.next_button();
        self.next_avatar();
    }
}

fn next_target(demo: &mut MorphDemo, targets: &[MaterialShape]) {
    let current = demo.to();
    let index = targets.iter().position(|s| *s == current).unwrap_or(0);
    demo.set_to(targets[(index + 1) % targets.len()]);
}

/// A card, a button and an avatar, all shaped by the morph at their current progress.
pub fn morph_surfaces(s: &MorphSurfaces, cx: &mut Cx<MorphSurfaces>) -> El {
    let dark = cx.system_dark();
    let t = tok::theme(dark);
    // Fixed hues keep each surface recognisable as it changes shape.
    let card_color = tok::accent(4, dark);
    let button_color = tok::accent(3, dark);
    let avatar_color = tok::accent(7, dark);
    let playing = s.playing();

    let card_shape = morph_paint(
        s.card.morph(),
        s.card.eased_progress(),
        PaintStyle::Soft,
        card_color,
    );
    let button_shape = morph_paint(
        s.button.morph(),
        s.button.eased_progress(),
        PaintStyle::Filled,
        button_color,
    );
    let avatar_shape = morph_paint(
        s.avatar.morph(),
        s.avatar.eased_progress(),
        PaintStyle::Filled,
        avatar_color,
    );

    col()
        .key("morph-surfaces")
        .w_full()
        .padding(28.0)
        .gap(22.0)
        .fill(t.bg)
        .child(
            col()
                .gap(6.0)
                .child(
                    text("Shape-shifting surfaces")
                        .font(TextStyle::new(29.0, 650.0))
                        .color(t.text),
                )
                .child(
                    text("Morphing card, button and avatar backgrounds, built from ordinary Rune elements.")
                        .font(type_::BODY)
                        .color(t.text_secondary),
                ),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .child(surface_action(
                    "morph-surfaces-play",
                    if playing { "Pause" } else { "Play" },
                    t,
                    card_color,
                    true,
                    MorphSurfaces::toggle_all,
                ))
                .child(surface_action(
                    "morph-surfaces-next",
                    "Next shapes",
                    t,
                    card_color,
                    false,
                    MorphSurfaces::next_shapes,
                )),
        )
        .child(
            col()
                .key("morph-surface-card")
                .w_full()
                .h(246.0)
                .padding(38.0)
                .gap(8.0)
                .justify_center()
                .shape(card_shape)
                .role("group")
                .label("Morphing card")
                .on_press(|s: &mut MorphSurfaces| s.next_card())
                .child(
                    text(format!(
                        "{} → {}",
                        s.card.from().label(),
                        s.card.to().label()
                    ))
                    .font(type_::LABEL)
                    .color(t.text_secondary),
                )
                .child(
                    text("Make room for expression")
                        .font(TextStyle::new(27.0, 650.0))
                        .color(t.text),
                )
                .child(
                    text("This card's background is a Material shape at the current frame of its morph. Click it to cycle the shape.")
                        .font(type_::BODY)
                        .color(t.text_secondary)
                        .wrap(),
                ),
        )
        .child(
            row()
                .w_full()
                .gap(28.0)
                .align(Align::Center)
                .child(
                    col()
                        .gap(8.0)
                        .child(
                            row()
                                .key("morph-surface-button")
                                .role("button")
                                .label("Morphing button")
                                .h(52.0)
                                .px(26.0)
                                .center()
                                .press_scale(0.96)
                                .shape(button_shape)
                                .on_press(|s: &mut MorphSurfaces| s.next_button())
                                .child(
                                    text("Continue")
                                        .font(type_::BODY)
                                        .color(Color::WHITE),
                                ),
                        )
                        .child(
                            text(format!(
                                "Morphing button · {} → {}",
                                s.button.from().label(),
                                s.button.to().label()
                            ))
                            .font(type_::LABEL)
                            .color(t.text_secondary),
                        ),
                )
                .child(
                    col()
                        .gap(8.0)
                        .align(Align::Center)
                        .child(
                            el()
                                .key("morph-surface-avatar")
                                .role("button")
                                .label("Morphing avatar")
                                .size(96.0, 96.0)
                                .center()
                                .press_scale(0.96)
                                .shape(avatar_shape)
                                .on_press(|s: &mut MorphSurfaces| s.next_avatar())
                                .child(
                                    text("AM")
                                        .font(TextStyle::new(28.0, 700.0))
                                        .color(Color::WHITE),
                                ),
                        )
                        .child(
                            text(format!(
                                "{} → {}",
                                s.avatar.from().label(),
                                s.avatar.to().label()
                            ))
                            .font(type_::LABEL)
                            .color(t.text_secondary),
                        ),
                ),
        )
}

fn surface_action(
    key: &'static str,
    label: &'static str,
    t: tok::Theme,
    accent: Color,
    primary: bool,
    run: impl Fn(&mut MorphSurfaces) + 'static,
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
