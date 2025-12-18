use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

#[derive(Deserialize, Debug)]
pub struct ThemeConfig {
    pub name: String,
    pub path: PathBuf,
    pub build_cmd: String,
    pub pre_build_cmd: Option<String>,
    pub css_path: PathBuf,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct ThemesConfig {
    pub theme: Vec<ThemeConfig>,
}

pub fn process_themes(themes_config_path: &Path, web_style_dir: &Path) -> Result<()> {
    info!(
        "Processing themes configuration from {:?}",
        themes_config_path
    );

    let content = fs::read_to_string(themes_config_path)
        .context("Failed to read themes configuration file")?;

    let config: ThemesConfig =
        toml::from_str(&content).context("Failed to parse themes configuration")?;

    // Validate duplicate names
    let mut names = HashSet::new();
    for theme in &config.theme {
        if !names.insert(&theme.name) {
            bail!("Duplicate theme name found: {}", theme.name);
        }
    }

    // Ensure output base directory exists
    if !web_style_dir.exists() {
        fs::create_dir_all(web_style_dir)?;
    }

    let base_dir = themes_config_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    // Process themes in parallel
    config.theme.par_iter().for_each(|theme| {
        let theme_dir = base_dir.join(&theme.path);
        let theme_output_dir = web_style_dir.join(&theme.name);

        // Create theme specific output directory
        if let Err(e) = fs::create_dir_all(&theme_output_dir) {
            warn!(
                "Failed to create output directory for theme '{}': {}",
                theme.name, e
            );
            return;
        }

        info!("Building theme '{}' in {:?}", theme.name, theme_dir);

        // Run pre-build command if exists
        if let Some(cmd) = &theme.pre_build_cmd {
            info!("Running pre-build command for theme '{}'", theme.name);
            let pre_status = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(&["/C", cmd])
                    .current_dir(&theme_dir)
                    .status()
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(&theme_dir)
                    .status()
            };

            match pre_status {
                Ok(s) if !s.success() => {
                    warn!(
                        "Theme '{}' pre-build command failed with status: {}",
                        theme.name, s
                    );
                    return; // Skip build if pre-build fails
                }
                Ok(_) => {
                    info!(
                        "Theme '{}' pre-build command finished successfully",
                        theme.name
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to execute pre-build command for theme '{}': {}",
                        theme.name, e
                    );
                    return;
                }
            }
        }

        // Run build command
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", &theme.build_cmd])
                .current_dir(&theme_dir)
                .status()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&theme.build_cmd)
                .current_dir(&theme_dir)
                .status()
        };

        match status {
            Ok(s) if s.success() => {
                info!("Theme '{}' built successfully", theme.name);

                // Copy generated files
                let source_dir = theme_dir.join(&theme.css_path);

                for file_name in &theme.files {
                    let source_file = source_dir.join(file_name);
                    let dest_file = theme_output_dir.join(file_name);

                    if let Err(e) = fs::copy(&source_file, &dest_file) {
                        warn!(
                            "Failed to copy file '{}' for theme '{}' from {:?} to {:?}: {}",
                            file_name, theme.name, source_file, dest_file, e
                        );
                    } else {
                        info!(
                            "Copied '{}' for theme '{}' to {:?}",
                            file_name, theme.name, dest_file
                        );
                    }
                }
            }
            Ok(s) => {
                warn!("Theme '{}' build failed with status: {}", theme.name, s);
            }
            Err(e) => {
                warn!(
                    "Failed to execute build command for theme '{}': {}",
                    theme.name, e
                );
            }
        }
    });

    Ok(())
}
