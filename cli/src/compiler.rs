use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::{Matter, ParsedEntity, Pod};
use pulldown_cmark::{Options, Parser};
use rayon::prelude::*;
use serde::Deserialize;
use sinter_core::{Post, PostMetadata, SiteData};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct SiteConfig {
    pub title: String,
    pub subtitle: String,
    pub description: String,
}

pub fn compile(input_dir: &Path, output_dir: &Path, config_path: &Path) -> Result<()> {
    info!("Scanning directory: {:?}", input_dir);

    // Load config
    let config = load_config(config_path)?;
    info!("Loaded configuration: {:?}", config);

    // 1. Load and parse posts
    let posts_with_path = load_posts(input_dir);
    info!("Successfully processed {} posts.", posts_with_path.len());

    // 2. Write individual post JSON files
    for (post, path) in &posts_with_path {
        let target_path = output_dir.join(path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent dirs for post")?;
        }

        let json = serde_json::to_string(post).context("Failed to serialize post")?;
        fs::write(&target_path, json).context("Failed to write post json")?;
    }
    info!(
        "Written {} individual post JSON files.",
        posts_with_path.len()
    );

    // 3. Build site data (metadata only)
    let site_data = build_site_data(&posts_with_path, config);

    // 4. Write site data
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    }

    let output_path = output_dir.join("site_data.json");
    let json = serde_json::to_string(&site_data).context("Failed to serialize site data")?;

    fs::write(&output_path, json).context("Failed to write site_data.json")?;

    info!("Compilation complete. Data written to {:?}", output_path);

    Ok(())
}
fn load_config(path: &Path) -> Result<SiteConfig> {
    if !path.exists() {
        anyhow::bail!("Config file not found: {:?}", path);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;
    let config: SiteConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {:?}", path))?;

    Ok(config)
}

fn load_posts(input_dir: &Path) -> Vec<(Post, String)> {
    let entries: Vec<_> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .collect();

    info!("Found {} markdown files.", entries.len());

    entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let relative_path = path.strip_prefix(input_dir).unwrap_or(path);

            let mut dest_rel_path = std::path::PathBuf::from("posts");
            dest_rel_path.push(relative_path);
            dest_rel_path.set_extension("json");

            let dest_path_str = dest_rel_path.to_string_lossy().replace('\\', "/");

            match fs::read_to_string(path) {
                Ok(content) => match parse_post(&content) {
                    Ok(post) => Some((post, dest_path_str)),
                    Err(e) => {
                        error!("Failed to parse file {:?}: {:?}", path, e);
                        None
                    }
                },
                Err(e) => {
                    error!("Failed to read file {:?}: {:?}", path, e);
                    None
                }
            }
        })
        .collect()
}

fn parse_post(content: &str) -> Result<Post> {
    // Parse Frontmatter
    let matter = Matter::<YAML>::new();
    let result: ParsedEntity<Pod> = matter
        .parse(content)
        .context("Failed to parse frontmatter")?;

    let metadata: PostMetadata = result
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing frontmatter"))?
        .deserialize()
        .context("Failed to deserialize frontmatter")?;

    // Parse Markdown to AST
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&result.content, options);
    let ast = markdown_parser::parse(parser);

    Ok(Post {
        metadata,
        content_ast: ast,
    })
}

mod markdown_parser;

fn build_site_data(posts: &[(Post, String)], config: SiteConfig) -> SiteData {
    let mut posts_map = HashMap::new();
    let mut tags_index = HashMap::new();

    for (post, path) in posts {
        for tag in &post.metadata.tags {
            tags_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(post.metadata.slug.clone());
        }

        use sinter_core::SitePostMetadata;

        let site_meta = SitePostMetadata {
            metadata: post.metadata.clone(),
            path: path.clone(),
        };

        if let Some(existing) = posts_map.insert(post.metadata.slug.clone(), site_meta) {
            warn!(
                "Duplicate slug found: {}. Overwriting previous post via {:?}.",
                existing.metadata.slug, existing.metadata.title
            );
        }
    }

    SiteData {
        generated_at: chrono::Utc::now(),
        posts: posts_map,
        tags_index,
        title: config.title,
        subtitle: config.subtitle,
        description: config.description,
    }
}

#[cfg(test)]
mod tests {
    use sinter_core::ContentNode;

    use super::*;

    #[test]
    fn test_parse_post_success() {
        let content = r#"---
id: "1"
title: "Test Post"
slug: "test-post"
date: "2023-01-01"
tags: ["rust", "test"]
summary: "A summary"
---
# Hello World
This is a test."#;

        let post = parse_post(content).expect("Failed to parse post");

        assert_eq!(post.metadata.title, "Test Post");
        assert_eq!(post.metadata.slug, "test-post");
        // Verify AST structure
        // Root -> [Heading, Paragraph]
        assert!(matches!(
            post.content_ast[0],
            ContentNode::Heading { level: 1, .. }
        ));
        assert!(matches!(post.content_ast[1], ContentNode::Paragraph { .. }));
    }

    #[test]
    fn test_parse_post_missing_frontmatter() {
        let content = "# Just Markdown";
        let result = parse_post(content);
        assert!(result.is_err());
    }
}
