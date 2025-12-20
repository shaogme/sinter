use sinter_ui::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{HtmlAnchorElement, Url};

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Home,
    Archives,
    Post(String),
    ArchivePost(String),
    NotFound,
}

impl Route {
    fn from_path(path: &str) -> Self {
        if path == "/" || path == "/index.html" {
            Route::Home
        } else if path == "/archives" || path == "/archives/" {
            Route::Archives
        } else if let Some(slug) = path.strip_prefix("/posts/") {
            let slug = slug.trim_matches('/');
            if slug.is_empty() {
                Route::NotFound
            } else {
                Route::Post(slug.to_string())
            }
        } else if let Some(slug) = path.strip_prefix("/archives/posts/") {
            let slug = slug.trim_matches('/');
            if slug.is_empty() {
                Route::NotFound
            } else {
                Route::ArchivePost(slug.to_string())
            }
        } else {
            Route::NotFound
        }
    }
}

pub fn use_router() -> (ReadSignal<Route>, ReadSignal<usize>) {
    let (path, set_path) = create_signal(
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_else(|| "/".to_string()),
    );

    let (search, set_search) = create_signal(
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default(),
    );

    // Sync UI on history change (PopState)
    create_effect(move || {
        let set_path = set_path;
        let set_search = set_search;
        let callback = Closure::wrap(Box::new(move |_| {
            if let Some(w) = web_sys::window() {
                let loc = w.location();
                let _ = set_path.set(loc.pathname().unwrap_or_default());
                let _ = set_search.set(loc.search().unwrap_or_default());
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        let window = web_sys::window().unwrap();
        let _ =
            window.add_event_listener_with_callback("popstate", callback.as_ref().unchecked_ref());

        on_cleanup(move || {
            let _ = window
                .remove_event_listener_with_callback("popstate", callback.as_ref().unchecked_ref());
        });
    });

    // Intercept <a> clicks for client-side routing
    create_effect(move || {
        let set_path = set_path;
        let set_search = set_search;

        let callback = Closure::wrap(Box::new(move |ev: web_sys::Event| {
            let target = ev.target().unwrap();
            let anchor = if let Some(a) = target.dyn_ref::<HtmlAnchorElement>() {
                Some(a.clone())
            } else {
                target
                    .unchecked_ref::<web_sys::Element>()
                    .closest("a")
                    .ok()
                    .flatten()
                    .and_then(|el| el.dyn_into::<HtmlAnchorElement>().ok())
            };

            if let Some(a) = anchor {
                let href = a.href();
                if let Ok(url) = Url::new(&href) {
                    // Check if it's the same origin
                    if let Ok(origin) = web_sys::window().unwrap().location().origin() {
                        if url.origin() == origin {
                            ev.prevent_default();
                            let pathname = url.pathname();
                            let search_str = url.search();

                            if let Ok(history) = web_sys::window().unwrap().history() {
                                let _ = history.push_state_with_url(
                                    &wasm_bindgen::JsValue::NULL,
                                    "",
                                    Some(&href),
                                );
                            }

                            let _ = set_path.set(pathname);
                            let _ = set_search.set(search_str);
                            web_sys::window().unwrap().scroll_to_with_x_and_y(0.0, 0.0);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        let window = web_sys::window().unwrap();
        let _ = window.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref());

        on_cleanup(move || {
            let _ = window
                .remove_event_listener_with_callback("click", callback.as_ref().unchecked_ref());
        });
    });

    let current_route = create_memo(move || Route::from_path(&path.get().unwrap_or_default()));

    let current_page = create_memo(move || {
        let s = search.get().unwrap_or_default();
        web_sys::UrlSearchParams::new_with_str(&s)
            .ok()
            .and_then(|p| p.get("page"))
            .and_then(|p_str| p_str.parse::<usize>().ok())
            .unwrap_or(1)
    });

    (current_route, current_page)
}
