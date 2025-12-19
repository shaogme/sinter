use leptos::prelude::*;

#[component]
pub fn Layout(children: ChildrenFn) -> impl IntoView {
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("GlobalState missing");

    let site_meta_signal = Signal::derive(move || state.site_meta.get().and_then(|r| r.ok()));

    let children = StoredValue::new(children);

    move || {
        let current_theme = state.theme.get();
        // create a fresh FnOnce closure that calls the stored Fn
        let children_closure = Box::new(move || children.with_value(|c| c()));
        current_theme.render_layout(children_closure, site_meta_signal)
    }
}
