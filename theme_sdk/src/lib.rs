use gloo_net::http::Request;
use leptos::prelude::*;
use sinter_core::{PageData, Post, SiteMetaData};
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlLinkElement, window};

pub trait Theme: Send + Sync + std::fmt::Debug {
    fn render_home(&self) -> AnyView;
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

    pub fn switch_theme(&self, name: &str) -> Option<Arc<dyn Theme>> {
        // 1. Get the requested theme
        let theme = self.get_theme(name)?;

        // 2. Load CSS dynamically
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");

        let link_id = "theme-css";
        let link_el = document.get_element_by_id(link_id);

        let url = format!("/themes/{}/default.css", name);
        leptos::logging::log!("Switching theme CSS to: {}", url);

        if let Some(link) = link_el {
            match link.dyn_into::<HtmlLinkElement>() {
                Ok(link) => {
                    link.set_href(&url);
                }
                Err(_) => {
                    leptos::logging::error!(
                        "Element with id 'theme-css' is not an HtmlLinkElement!"
                    );
                }
            }
        } else {
            let head = document.head().expect("document should have a head");
            let link = document
                .create_element("link")
                .expect("failed to create link element");
            let link: HtmlLinkElement = link.unchecked_into();
            link.set_rel("stylesheet");
            link.set_href(&url);
            link.set_id(link_id);
            if let Err(e) = head.append_child(&link) {
                leptos::logging::error!("Failed to append child: {:?}", e);
            }
        }

        // 3. Return the theme so the app can update its state
        Some(theme)
    }
}

pub async fn fetch_site_meta() -> Result<SiteMetaData, String> {
    let resp = Request::get("/sinter_data/site_data.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!(
            "Failed to fetch site data: {} {}",
            resp.status(),
            resp.status_text()
        ));
    }

    resp.json::<SiteMetaData>()
        .await
        .map_err(|e| format!("JSON Parse Error: {}", e))
}

pub async fn fetch_page_data(page: usize) -> Result<PageData, String> {
    let url = format!("/sinter_data/pages/page_{}.json", page);
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!(
            "Failed to fetch page data: {} {}",
            resp.status(),
            resp.status_text()
        ));
    }

    resp.json::<PageData>()
        .await
        .map_err(|e| format!("JSON Parse Error: {}", e))
}

#[derive(Clone)]
pub struct GlobalState {
    pub site_meta: LocalResource<Result<SiteMetaData, String>>,
    pub theme: RwSignal<Arc<dyn Theme>>,
    pub manager: Arc<ThemeManager>,
}

impl GlobalState {
    pub fn new(manager: Arc<ThemeManager>, initial_theme_name: &str) -> Self {
        let theme_instance = manager
            .switch_theme(initial_theme_name)
            .expect("Initial theme not found");

        Self {
            site_meta: LocalResource::new(fetch_site_meta),
            theme: RwSignal::new(theme_instance),
            manager,
        }
    }

    pub fn switch_theme(&self, name: &str) {
        if let Some(new_theme) = self.manager.switch_theme(name) {
            self.theme.set(new_theme);
        } else {
            leptos::logging::warn!("Theme '{}' not found", name);
        }
    }
}

// Hooks

pub fn use_site_meta() -> Option<Result<SiteMetaData, String>> {
    use_context::<GlobalState>().and_then(|state| state.site_meta.get())
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
