use crate::app::PostParams;

use leptos::prelude::*;
use leptos_router::hooks::{use_params, use_query_map};
use sinter_core::Post;
use sinter_theme_sdk::{PageDataContext, fetch_archive_page_data, fetch_json, fetch_page_data};

#[component]
pub fn Home() -> impl IntoView {
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("缺少 GlobalState");
    let query = use_query_map();

    // 获取当前页码
    let page = move || {
        query
            .get()
            .get("page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1)
    };

    // 创建页面数据资源
    let page_data_resource = LocalResource::new(move || {
        let page_num = page();
        async move { fetch_page_data(page_num).await }
    });

    // 提供 PageDataContext 供主题使用
    provide_context(PageDataContext(page_data_resource));

    view! {
        {state.theme.get_untracked().render_home()}
    }
}

#[component]
pub fn Archives() -> impl IntoView {
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("缺少 GlobalState");
    let query = use_query_map();

    // 获取当前页码
    let page = move || {
        query
            .get()
            .get("page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1)
    };

    // 创建页面数据资源 (Archives)
    let page_data_resource = LocalResource::new(move || {
        let page_num = page();
        async move { fetch_archive_page_data(page_num).await }
    });

    // 提供 PageDataContext 供主题使用
    provide_context(PageDataContext(page_data_resource));

    view! {
        {state.theme.get_untracked().render_archive()}
    }
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("缺少 GlobalState");

    // 捕获 state 以便在闭包中使用
    let theme_signal = state.theme;

    // 根据 slug 获取文章详情
    let post_resource = LocalResource::new(move || {
        let slug = params
            .get()
            .map(|p| p.slug)
            .unwrap_or(None)
            .unwrap_or_default();

        async move {
            // 直接构建 URL 请求文章数据
            let url = format!("/sinter_data/posts/{}.json", slug);

            match fetch_json::<Post>(&url).await {
                Ok(post) => Some(post),
                Err(_) => None,
            }
        }
    });

    view! {
        <Suspense fallback=move || theme_signal.get_untracked().render_post_loading()>
            {move || {
                let theme = theme_signal.get_untracked();
                match post_resource.get() {
                    Some(Some(post)) => theme.render_post(post),
                    Some(None) => theme.render_post_not_found(),
                    None => theme.render_post_loading(),
                }
            }}
        </Suspense>
    }
}

#[component]
pub fn ArchivePostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let state = use_context::<sinter_theme_sdk::GlobalState>().expect("缺少 GlobalState");

    let theme_signal = state.theme;

    let post_resource = LocalResource::new(move || {
        let slug = params
            .get()
            .map(|p| p.slug)
            .unwrap_or(None)
            .unwrap_or_default();

        async move {
            let url = format!("/sinter_data/archives/{}.json", slug);

            match fetch_json::<Post>(&url).await {
                Ok(post) => Some(post),
                Err(_) => None,
            }
        }
    });

    view! {
        <Suspense fallback=move || theme_signal.get_untracked().render_post_loading()>
            {move || {
                let theme = theme_signal.get_untracked();
                match post_resource.get() {
                    Some(Some(post)) => theme.render_post(post),
                    Some(None) => theme.render_post_not_found(),
                    None => theme.render_post_loading(),
                }
            }}
        </Suspense>
    }
}
