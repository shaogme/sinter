use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl fmt::Display for LiteDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Serialize for LiteDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LiteDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(serde::de::Error::custom("Expected format YYYY-MM-DD"));
        }
        let year = parts[0].parse().map_err(serde::de::Error::custom)?;
        let month = parts[1].parse().map_err(serde::de::Error::custom)?;
        let day = parts[2].parse().map_err(serde::de::Error::custom)?;

        Ok(LiteDate { year, month, day })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PostMetadata {
    pub id: String,
    pub title: String,
    pub slug: String,

    pub date: LiteDate,

    #[serde(default)]
    pub tags: Vec<String>,

    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SitePostMetadata {
    #[serde(flatten)]
    pub metadata: PostMetadata,
    pub path: String, // Relative path to the generated JSON file
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentNode {
    // Container nodes
    Paragraph {
        children: Vec<ContentNode>,
    },
    Heading {
        level: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        classes: Vec<String>,
        children: Vec<ContentNode>,
    },
    List {
        ordered: bool,
        children: Vec<ContentNode>,
    },
    ListItem {
        children: Vec<ContentNode>,
    },
    BlockQuote {
        children: Vec<ContentNode>,
    },

    // Leaf nodes
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    Text {
        value: String,
    },
    Html {
        value: String,
    },
    Math {
        value: String,
        display: bool,
    },
    TaskListMarker {
        checked: bool,
    },
    ThematicBreak,

    // Inline formatting
    Emphasis {
        children: Vec<ContentNode>,
    },
    Strong {
        children: Vec<ContentNode>,
    },
    Strikethrough {
        children: Vec<ContentNode>,
    },

    // Links & Images
    Link {
        url: String,
        title: Option<String>,
        children: Vec<ContentNode>,
    },
    Image {
        url: String,
        title: Option<String>,
        alt: String,
    },

    // Table
    Table {
        children: Vec<ContentNode>,
    },
    TableHead {
        children: Vec<ContentNode>,
    },
    TableBody {
        children: Vec<ContentNode>,
    },
    TableRow {
        children: Vec<ContentNode>,
    },
    TableCell {
        children: Vec<ContentNode>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Post {
    #[serde(flatten)]
    pub metadata: PostMetadata,
    pub content_ast: Vec<ContentNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteMetaData {
    pub generated_at: String, // ISO String or similar
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
    pub total_pages: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageData {
    pub posts: Vec<SitePostMetadata>,
    pub tags_index: HashMap<String, Vec<String>>,
}

pub mod constants {
    pub const DEFAULT_POSTS_PER_PAGE: usize = 10;
    pub const SITE_DATA_FILENAME: &str = "site_data.json";
    pub const PAGES_DIR: &str = "pages";
    pub const POSTS_DIR: &str = "posts";
}
