//! Material 3 Expressive's shape library, browsable. Click a tile to preview its shape above the
//! grid; `⌘⇧D` (`Ctrl+Shift+D`) opens Rune's playground, where every category is its own story, the
//! Morph story animates any shape into any other, the Surfaces story morphs a card, button and
//! avatar, and variants 1–3 switch between filled, outlined and soft paint.
//!
//! ```sh
//! cargo run --example material-shapes
//! cargo run --example material-shapes -- --playground "Material 3/All shapes"
//! cargo run --example material-shapes -- --playground "Material 3/Morph"
//! cargo run --example material-shapes -- --playground "Material 3/Surfaces"
//! cargo run --example material-shapes -- --snap shapes.png
//! ```

use rune::tok;
use rune::{App, AppSpec, Cx, El, ScrollSpec, col};
use rune_material_shapes::{Gallery, gallery, playground};

fn view(s: &Gallery, cx: &mut Cx<Gallery>) -> El {
    let t = tok::theme(cx.system_dark());
    col()
        .key("material-shapes-app")
        .w_full()
        .h_full()
        .fill(t.bg)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(gallery(s, cx))
}

fn main() {
    let spec = AppSpec::new("Material 3 shapes", (1100.0, 860.0), Gallery::new, view);
    App::new(spec).playground(playground()).run();
}
