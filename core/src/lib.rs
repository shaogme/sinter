use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PostMetadata {
    pub id: String,
    pub title: String,
    pub slug: String,

    #[serde(with = "date_format")]
    pub date: NaiveDate,

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
pub struct SiteData {
    pub generated_at: DateTime<Utc>,
    pub posts: HashMap<String, SitePostMetadata>,
    pub tags_index: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}
