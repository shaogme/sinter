use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    params::Params,
    *,
};

use crate::components::Layout;
use crate::pages::Home;
use crate::pages::PostView;

#[derive(PartialEq, Params, Clone, Debug)]
pub struct PostParams {
    pub slug: Option<String>,
}

use sinter_theme_sdk::GlobalState;
use std::sync::Arc;

#[component]
pub fn App() -> impl IntoView {
    // 0. Initialize themes registry
    let manager = sinter_themes::init_manager();
    let manager = Arc::new(manager);

    // 1. Create the GlobalState which includes data fetching resources and theme
    // 2. Provide the state as global context
    provide_context(GlobalState::new(manager, "default"));

    view! {
        <Router>
            <Layout>
                <Suspense fallback=move || {
                    let state = use_context::<GlobalState>().expect("GlobalState missing");
                    state.theme.get_untracked().render_loading()
                }>
                    <Routes fallback=|| view! { "404 - Not Found" }>
                        <Route path=path!("/") view=Home />
                        <Route path=path!("/posts/:slug") view=PostView />
                    </Routes>
                </Suspense>
            </Layout>
        </Router>
    }
}
