use crate::app::PostParams;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::{use_params, use_query_map};
use sinter_core::Post;
use sinter_theme_sdk::{PageDataContext, fetch_page_data};

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
        {move || state.theme.get().render_home()}
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
        }
    });

    view! {
        <Suspense fallback=move || theme_signal.get().render_post_loading()>
            {move || {
                let theme = theme_signal.get();
                match post_resource.get() {
                    Some(Some(post)) => theme.render_post(post),
                    Some(None) => theme.render_post_not_found(),
                    None => theme.render_post_loading(),
                }
            }}
        </Suspense>
    }
}
