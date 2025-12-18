use gloo_net::http::Request;
use leptos::prelude::*;
use sinter_core::{Post, SiteData, SitePostMetadata};
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlLinkElement, window};

pub trait Theme: Send + Sync + std::fmt::Debug {
    fn render_home(&self, posts: Vec<SitePostMetadata>, site_data: &SiteData) -> AnyView;
    fn render_post(&self, post: Post) -> AnyView;
    fn render_post_loading(&self) -> AnyView;
    fn render_loading(&self) -> AnyView;
    fn render_post_not_found(&self) -> AnyView;
    fn render_error(&self, message: String) -> AnyView;
    fn render_layout(&self, children: Children, site_data: Signal<Option<SiteData>>) -> AnyView;
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
        let head = document.head().expect("document should have a head");

        let link_id = "theme-css";
        let link_el = document.get_element_by_id(link_id);

        let url = format!("themes/{}/default.css", name);

        if let Some(link) = link_el {
            let link: HtmlLinkElement = link.unchecked_into();
            link.set_href(&url);
        } else {
            let link = document.create_element("link").unwrap();
            let link: HtmlLinkElement = link.unchecked_into();
            link.set_rel("stylesheet");
            link.set_href(&url);
            link.set_id(link_id);
            head.append_child(&link).unwrap();
        }

        // 3. Return the theme so the app can update its state
        Some(theme)
    }
}

pub async fn fetch_site_data() -> Result<SiteData, String> {
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

    resp.json::<SiteData>()
        .await
        .map_err(|e| format!("JSON Parse Error: {}", e))
}

#[derive(Clone)]
pub struct GlobalState {
    pub site_data: LocalResource<Result<SiteData, String>>,
    pub theme: RwSignal<Arc<dyn Theme>>,
    pub manager: Arc<ThemeManager>,
}

impl GlobalState {
    // New constructor requires an initialized ThemeManager
    pub fn new(manager: Arc<ThemeManager>, initial_theme_name: &str) -> Self {
        let theme_instance = manager
            .switch_theme(initial_theme_name)
            .expect("Initial theme not found");

        Self {
            site_data: LocalResource::new(fetch_site_data),
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
