use anyhow::{Context, Result};
use gray_matter::engine::YAML;
use gray_matter::{Matter, ParsedEntity, Pod};
use pulldown_cmark::{Options, Parser};
use rayon::prelude::*;
use serde::Deserialize;
use sinter_core::constants::{DEFAULT_POSTS_PER_PAGE, PAGES_DIR, SITE_DATA_FILENAME};
use sinter_core::{PageData, Post, PostMetadata, SiteMetaData, SitePostMetadata};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct SiteConfig {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub posts_per_page: Option<usize>,
}

pub fn compile(input_dir: &Path, output_dir: &Path, config_path: &Path) -> Result<()> {
    info!("Starting compilation...");
    info!("Input directory: {:?}", input_dir);

    // 1. Initialization
    let config = load_config(config_path)?;
    info!("Configuration loaded: {:?}", config);
    let posts_per_page = config.posts_per_page.unwrap_or(DEFAULT_POSTS_PER_PAGE);

    let temp_dir = tempfile::Builder::new()
        .prefix("sinter_build")
        .tempdir()
        .context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();
    info!("Temporary directory created at: {:?}", temp_path);

    // 2. Process Posts
    let mut posts = load_all_posts(input_dir);
    posts.sort_by(|a, b| b.0.metadata.date.cmp(&a.0.metadata.date));
    info!("Processed {} posts.", posts.len());

    // 3. Generation
    write_post_files(&posts, temp_path)?;
    generate_pages(&posts, temp_path, posts_per_page)?;
    write_site_metadata(posts.len(), &config, posts_per_page, temp_path)?;

    // 4. Deployment
    deploy_to_output(temp_path, output_dir)?;

    info!("Compilation finished successfully!");
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

fn load_all_posts(input_dir: &Path) -> Vec<(Post, String)> {
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

            // Construct the destination path for the JSON file
            let mut dest_rel_path = PathBuf::from("posts");
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

fn write_post_files(posts: &[(Post, String)], output_dir: &Path) -> Result<()> {
    for (post, rel_path) in posts {
        let target_path = output_dir.join(rel_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent dirs for post")?;
        }

        let json = serde_json::to_string(post).context("Failed to serialize post")?;
        fs::write(&target_path, json).context("Failed to write post json")?;
    }
    info!("Written {} individual post JSON files.", posts.len());
    Ok(())
}

fn generate_pages(
    posts: &[(Post, String)],
    output_dir: &Path,
    posts_per_page: usize,
) -> Result<()> {
    let pages_dir = output_dir.join(PAGES_DIR);
    fs::create_dir_all(&pages_dir).context("Failed to create pages directory")?;

    for (i, chunk) in posts.chunks(posts_per_page).enumerate() {
        let page_num = i + 1;
        let mut page_posts = Vec::new();
        let mut tags_index = HashMap::new();

        for (post, path) in chunk {
            let site_meta = SitePostMetadata {
                metadata: post.metadata.clone(),
                path: path.clone(),
            };
            page_posts.push(site_meta);

            for tag in &post.metadata.tags {
                tags_index
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(post.metadata.slug.clone());
            }
        }

        let page_data = PageData {
            posts: page_posts,
            tags_index,
        };

        let page_json =
            serde_json::to_string(&page_data).context("Failed to serialize page data")?;
        fs::write(pages_dir.join(format!("page_{}.json", page_num)), page_json)
            .context("Failed to write page json")?;
    }

    let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;
    info!("Generated {} pages in {:?}", total_pages, pages_dir);

    Ok(())
}

fn write_site_metadata(
    total_posts: usize,
    config: &SiteConfig,
    posts_per_page: usize,
    output_dir: &Path,
) -> Result<()> {
    let total_pages = if total_posts == 0 {
        0
    } else {
        (total_posts + posts_per_page - 1) / posts_per_page
    };

    let site_meta = SiteMetaData {
        generated_at: chrono::Utc::now(),
        title: config.title.clone(),
        subtitle: config.subtitle.clone(),
        description: config.description.clone(),
        total_pages,
    };

    let output_path = output_dir.join(SITE_DATA_FILENAME);
    let json = serde_json::to_string(&site_meta).context("Failed to serialize site metadata")?;
    fs::write(&output_path, json).context("Failed to write site metadata file")?;

    info!("Site metadata written to {:?}", output_path);
    Ok(())
}

fn deploy_to_output(temp_path: &Path, output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).context("Failed to create final output directory")?;
    }

    // Helper for recursive copy
    fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
        for entry in WalkDir::new(src) {
            let entry = entry?;
            let path = entry.path();
            if path == src {
                continue;
            }

            let rel_path = path.strip_prefix(src)?;
            let target = dst.join(rel_path);

            if path.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(path, &target)?;
            }
        }
        Ok(())
    }

    copy_recursive(temp_path, output_dir)?;
    info!(
        "Content deployed from temporary directory to {:?}",
        output_dir
    );
    Ok(())
}

mod markdown_parser;

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
