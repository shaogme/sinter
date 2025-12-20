use sinter_theme_sdk::{Children, GlobalState};
use sinter_ui::dom::tag::*;
use sinter_ui::dom::view::{AnyView, IntoAnyView};
use sinter_ui::prelude::*;

pub fn layout(children: Children) -> AnyView {
    if let Some(state) = use_context::<GlobalState>() {
        // Use Dynamic to make the layout reactive to theme changes
        Dynamic::new(move || {
            let theme = state.theme.get().expect("Theme not found inside layout");

            let site_meta_signal = create_memo(move || state.site_meta.get().and_then(|r| r.ok()));

            let children_clone = children.clone();
            theme.render_layout(children_clone, site_meta_signal)
        })
        .into_any()
    } else {
        Dynamic::new(|| div().text("GlobalState missing").into_any()).into_any()
    }
}
