use leptos::prelude::*;
use sinter_core::{PageData, Post, SiteMetaData};
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlLinkElement, Response, window};

// Helper for fetching JSON
pub async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    let window = window().ok_or("No global window")?;
    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object".to_string())?;

    if !resp.ok() {
        return Err(format!(
            "Failed to fetch {}: {} {}",
            url,
            resp.status(),
            resp.status_text()
        ));
    }

    let json_value = JsFuture::from(resp.json().map_err(|e| format!("json error: {:?}", e))?)
        .await
        .map_err(|e| format!("json await error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json_value).map_err(|e| format!("Deserialization error: {}", e))
}

pub trait Theme: Send + Sync + std::fmt::Debug {
    fn render_home(&self) -> AnyView;
    fn render_archive(&self) -> AnyView;
    fn render_post(&self, post: Post) -> AnyView;
    fn render_post_loading(&self) -> AnyView;
    fn render_loading(&self) -> AnyView;
    fn render_post_not_found(&self) -> AnyView;
    fn render_error(&self, message: String) -> AnyView;
    fn render_layout(&self, children: Children, site_meta: Signal<Option<SiteMetaData>>)
    -> AnyView;
}

#[derive(Debug)]
pub struct ThemeManager {
    themes: HashMap<&'static str, Arc<dyn Theme>>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let themes: HashMap<&'static str, Arc<dyn Theme>> = HashMap::new();
        Self { themes }
    }

    pub fn get_theme(&self, name: &str) -> Option<Arc<dyn Theme>> {
        self.themes.get(name).cloned()
    }

    pub fn register_theme(&mut self, name: &'static str, theme: Arc<dyn Theme>) {
        self.themes.insert(name, theme);
    }

    pub fn get_available_themes(&self) -> Vec<&'static str> {
        self.themes.keys().cloned().collect()
    }

    pub async fn switch_theme(&self, name: &str) -> Option<Arc<dyn Theme>> {
        // 1. Get the requested theme
        let theme = self.get_theme(name)?;

        // 2. Load CSS dynamically with Double Buffering
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let head = document.head().expect("document should have a head");

        let url = format!("/themes/{}/default.css", name);
        leptos::logging::log!("Switching theme CSS to: {}", url);

        // Create new link
        let new_link = document
            .create_element("link")
            .expect("failed to create link element");
        let new_link: HtmlLinkElement = new_link.unchecked_into();
        new_link.set_rel("stylesheet");
        new_link.set_href(&url);

        // Prepare promise to wait for load
        let new_link_clone = new_link.clone();
        let doc_clone = document.clone();

        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let new_link_inner = new_link_clone.clone();
            let doc_inner = doc_clone.clone();

            let callback = wasm_bindgen::closure::Closure::once(move || {
                // Find and remove old link
                let old_link = doc_inner.get_element_by_id("theme-css");
                if let Some(old) = old_link {
                    old.remove();
                }
                // Adopt the ID for the new link
                new_link_inner.set_id("theme-css");

                // Notify completion
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            });

            new_link_clone.set_onload(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        });

        if let Err(e) = head.append_child(&new_link) {
            leptos::logging::error!("Failed to append child: {:?}", e);
            return None;
        }

        // Wait for CSS to load
        let _ = JsFuture::from(promise).await;

        // 3. Return the theme so the app can update its state
        Some(theme)
    }
}

pub async fn fetch_site_meta() -> Result<SiteMetaData, String> {
    fetch_json("/sinter_data/site_data.json").await
}

pub async fn fetch_page_data(page: usize) -> Result<PageData, String> {
    fetch_json(&format!("/sinter_data/pages/page_{}.json", page)).await
}

pub async fn fetch_archive_page_data(page: usize) -> Result<PageData, String> {
    fetch_json(&format!("/sinter_data/archives/pages/page_{}.json", page)).await
}

#[derive(Clone)]
pub struct GlobalState {
    pub site_meta: LocalResource<Result<SiteMetaData, String>>,
    pub theme: RwSignal<Arc<dyn Theme>>,
    pub manager: Arc<ThemeManager>,
}

impl GlobalState {
    pub fn new(manager: Arc<ThemeManager>, initial_theme_name: &str) -> Self {
        // Try to get theme from local storage
        let storage = window().and_then(|w| w.local_storage().ok()).flatten();
        let storage_theme = storage
            .as_ref()
            .and_then(|s| s.get_item("sinter_theme").ok())
            .flatten();
        let theme_name = storage_theme.as_deref().unwrap_or(initial_theme_name);

        let theme_instance = manager
            .get_theme(theme_name)
            .or_else(|| manager.get_theme(initial_theme_name))
            .expect("Initial theme not found");

        Self {
            site_meta: LocalResource::new(fetch_site_meta),
            theme: RwSignal::new(theme_instance),
            manager,
        }
    }

    pub fn switch_theme(&self, name: &str) {
        let manager = self.manager.clone();
        let theme_signal = self.theme;
        let name_owned = name.to_string();

        leptos::task::spawn_local(async move {
            if let Some(new_theme) = manager.switch_theme(&name_owned).await {
                theme_signal.set(new_theme);
                if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
                    let _ = storage.set_item("sinter_theme", &name_owned);
                }
            } else {
                leptos::logging::warn!("Theme '{}' not found", &name_owned);
            }
        });
    }
}

// Hooks

pub fn use_site_meta() -> Option<LocalResource<Result<SiteMetaData, String>>> {
    use_context::<GlobalState>().map(|state| state.site_meta)
}

// Ensure you provide this context in your page component!
#[derive(Clone, Copy)]
pub struct PageDataContext(pub LocalResource<Result<PageData, String>>);

pub fn use_page_data() -> Option<LocalResource<Result<PageData, String>>> {
    use_context::<PageDataContext>().map(|ctx| ctx.0)
}

#[derive(Clone, Copy)]
pub struct CurrentPageContext(pub Signal<usize>);

pub fn use_current_page() -> Signal<usize> {
    use_context::<CurrentPageContext>()
        .map(|c| c.0)
        .unwrap_or_else(|| Signal::derive(|| 1))
}
