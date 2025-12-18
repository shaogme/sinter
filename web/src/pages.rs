use crate::app::PostParams;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use sinter_core::Post;
use sinter_core::SitePostMetadata;

#[component]
pub fn Home() -> impl IntoView {
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("GlobalState missing");

    view! {
        {move || {
            state.site_data.get().map(|result| match result {
                Ok(site_data) => {
                    let mut posts: Vec<SitePostMetadata> = site_data.posts.values().cloned().collect();
                    posts.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));

                    state.theme.get().render_home(posts, &site_data)
                },
                Err(e) => state.theme.get().render_error(e),
            })
        }}
    }
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("GlobalState missing");
    let theme_fb = state.theme.clone();
    let theme_content = state.theme.clone();

    // Resource that depends on the slug/site_data and fetches the individual post
    let post_resource = LocalResource::new(move || {
        let slug = params
            .get()
            .map(|p| p.slug)
            .unwrap_or(None)
            .unwrap_or_default();
        let site_data_opt = state.site_data.get();

        async move {
            let meta = site_data_opt.and_then(|result| match result {
                Ok(data) => data.posts.get(&slug).cloned(),
                Err(_) => None,
            });

            if let Some(meta) = meta {
                let path = meta.path.clone();
                let url = format!("/sinter_data/{}", path);

                match Request::get(&url).send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            resp.json::<Post>().await.ok()
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    });

    view! {
        <Suspense fallback=move || theme_fb.get().render_post_loading()>
            {move || {
                match post_resource.get() {
                    Some(Some(post)) => theme_content.get().render_post(post),
                    Some(None) => theme_content.get().render_post_not_found(),
                    None => theme_content.get().render_post_loading(),
                }
            }}
        </Suspense>
    }
}
