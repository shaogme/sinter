use sinter_core::Post;
use sinter_theme_sdk::{
    GlobalState, PageDataContext, fetch_archive_page_data, fetch_json, fetch_page_data,
};
use sinter_ui::dom::suspense::suspense;
use sinter_ui::dom::tag::div;
use sinter_ui::dom::view::IntoAnyView;
use sinter_ui::prelude::*;

pub fn home(page: ReadSignal<usize>) -> impl IntoAnyView {
    if let Some(state) = use_context::<GlobalState>() {
        // Create page data resource
        let page_data_resource = create_resource(
            move || page.get().unwrap_or(1),
            |page_num| async move { fetch_page_data(page_num).await },
        )
        .expect("Failed to create resource");

        // Provide PageDataContext for the theme
        let _ = provide_context(PageDataContext(page_data_resource));

        // Render the theme's home page
        Dynamic::new(move || {
            let theme = state.theme.get().expect("Theme not found");
            theme.render_home()
        })
        .into_any()
    } else {
        Dynamic::new(|| div().text("GlobalState missing").into_any()).into_any()
    }
}

pub fn archives(page: ReadSignal<usize>) -> impl IntoAnyView {
    if let Some(state) = use_context::<GlobalState>() {
        // Create page data resource (Archives)
        let page_data_resource = create_resource(
            move || page.get().unwrap_or(1),
            |page_num| async move { fetch_archive_page_data(page_num).await },
        )
        .expect("Failed to create resource");

        // Provide PageDataContext for the theme
        let _ = provide_context(PageDataContext(page_data_resource));

        Dynamic::new(move || {
            let theme = state.theme.get().expect("Theme not found");
            theme.render_archive()
        })
        .into_any()
    } else {
        Dynamic::new(|| div().text("GlobalState missing").into_any()).into_any()
    }
}

pub fn post_view(slug: ReadSignal<String>) -> impl IntoAnyView {
    if let Some(state) = use_context::<GlobalState>() {
        let theme_signal = state.theme;

        // Fetch post details based on slug
        let post_resource = create_resource(
            move || slug.get().unwrap_or_default(),
            |current_slug| async move {
                if current_slug.is_empty() {
                    return None;
                }
                let url = format!("/sinter_data/posts/{}.json", current_slug);
                match fetch_json::<Post>(&url).await {
                    Ok(post) => Some(post),
                    Err(_) => None,
                }
            },
        )
        .expect("Failed to create resource");

        let theme_fallback = theme_signal;

        suspense()
            .fallback(move || {
                if let Some(theme) = theme_fallback.get() {
                    theme.render_post_loading()
                } else {
                    div().text("Loading...").into_any()
                }
            })
            .children(move || {
                let theme = theme_signal.get().expect("Theme not found");
                match post_resource.get() {
                    Some(Some(post)) => theme.render_post(post),
                    Some(None) => theme.render_post_not_found(),
                    None => theme.render_post_loading(),
                }
            })
            .into_any()
    } else {
        div().text("GlobalState missing").into_any()
    }
}

pub fn archive_post_view(slug: ReadSignal<String>) -> impl IntoAnyView {
    if let Some(state) = use_context::<GlobalState>() {
        let theme_signal = state.theme;

        let post_resource = create_resource(
            move || slug.get().unwrap_or_default(),
            |current_slug| async move {
                if current_slug.is_empty() {
                    return None;
                }
                let url = format!("/sinter_data/archives/{}.json", current_slug);
                match fetch_json::<Post>(&url).await {
                    Ok(post) => Some(post),
                    Err(_) => None,
                }
            },
        )
        .expect("Failed to create resource");

        let theme_fallback = theme_signal;

        suspense()
            .fallback(move || {
                if let Some(theme) = theme_fallback.get() {
                    theme.render_post_loading()
                } else {
                    div().text("Loading...").into_any()
                }
            })
            .children(move || {
                let theme = theme_signal.get().expect("Theme not found");
                match post_resource.get() {
                    Some(Some(post)) => theme.render_post(post),
                    Some(None) => theme.render_post_not_found(),
                    None => theme.render_post_loading(),
                }
            })
            .into_any()
    } else {
        div().text("GlobalState missing").into_any()
    }
}
