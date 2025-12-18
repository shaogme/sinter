use sinter_theme_default::DefaultTheme;
use sinter_theme_default_light::DefaultLightTheme;
use sinter_theme_sdk::ThemeManager;
use std::sync::Arc;

pub fn init_manager() -> ThemeManager {
    let mut manager = ThemeManager::new();
    manager.register_theme("default", Arc::new(DefaultTheme));
    manager.register_theme("default_light", Arc::new(DefaultLightTheme));
    manager
}
