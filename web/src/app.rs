use crate::components::layout;
use crate::pages::{archive_post_view, archives, home, post_view};
use crate::router::{Route, use_router};
use sinter_theme_sdk::GlobalState;
use sinter_ui::dom::tag::div;
use sinter_ui::dom::view::IntoAnyView;
use sinter_ui::prelude::*;
use std::sync::Arc;

pub fn app() -> impl IntoAnyView {
    // 0. Initialize themes registry
    let manager = sinter_themes::init_manager();
    let manager = Arc::new(manager);

    // 1. Create the GlobalState which includes data fetching resources and theme
    // 2. Provide the state as global context
    let _ = provide_context(GlobalState::new(manager, "default"));

    // 3. Use Simple Router
    let (route, page) = use_router();

    // 4. Create the view
    let content_fn = Arc::new(move || {
        let current_route = route.get().unwrap_or(Route::NotFound);
        let current_page = page;

        match current_route {
            Route::Home => home(current_page).into_any(),
            Route::Archives => archives(current_page).into_any(),
            Route::Post(slug_str) => {
                let slug_signal = create_memo(move || {
                    if let Some(Route::Post(s)) = route.get() {
                        s
                    } else {
                        // If route changed, this signal might be stale for a moment or re-evaluated.
                        // But since we are inside the effect re-run, route.get() is current.
                        slug_str.clone()
                    }
                });
                post_view(slug_signal).into_any()
            }
            Route::ArchivePost(slug_str) => {
                let slug_signal = create_memo(move || {
                    if let Some(Route::ArchivePost(s)) = route.get() {
                        s
                    } else {
                        slug_str.clone()
                    }
                });
                archive_post_view(slug_signal).into_any()
            }
            Route::NotFound => div().text("404 - Not Found").into_any(),
        }
    });

    layout(content_fn).into_any()
}
