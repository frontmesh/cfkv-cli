use crate::error::{BlogError, Result};
use regex::Regex;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Parsed markdown file with frontmatter and content
#[derive(Debug)]
pub struct ParsedMarkdown {
    pub metadata: BTreeMap<String, Value>,
    pub content: String,
}

/// Parser for markdown files with YAML frontmatter
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parse markdown content with YAML frontmatter
    pub fn parse(content: &str) -> Result<ParsedMarkdown> {
        // Regex to match frontmatter: ---\n(yaml)\n---\n(markdown)
        let regex = Regex::new(r"^---\n([\s\S]*?)\n---\n([\s\S]*)$")
            .map_err(|e| BlogError::FrontmatterError(e.to_string()))?;

        let captures = regex.captures(content).ok_or_else(|| {
            BlogError::FrontmatterError("Invalid markdown format: missing frontmatter".to_string())
        })?;

        let yaml_str = captures.get(1).unwrap().as_str();
        let markdown_content = captures.get(2).unwrap().as_str();

        // Parse YAML frontmatter
        let metadata: BTreeMap<String, Value> = serde_yaml::from_str(yaml_str)?;

        Ok(ParsedMarkdown {
            metadata,
            content: markdown_content.trim().to_string(),
        })
    }

    /// Extract a string value from metadata
    pub fn get_string(metadata: &BTreeMap<String, Value>, key: &str) -> Result<String> {
        metadata
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| BlogError::ValidationError(format!("Missing or invalid field: {}", key)))
    }

    /// Extract an optional string value from metadata
    pub fn get_optional_string(metadata: &BTreeMap<String, Value>, key: &str) -> Option<String> {
        metadata
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    }

    /// Extract a list of strings from metadata
    pub fn get_string_list(metadata: &BTreeMap<String, Value>, key: &str) -> Result<Vec<String>> {
        match metadata.get(key) {
            Some(Value::Sequence(seq)) => {
                let tags: Result<Vec<String>> = seq
                    .iter()
                    .map(|v| {
                        v.as_str().map(|s| s.to_string()).ok_or_else(|| {
                            BlogError::ValidationError("Invalid tag format".to_string())
                        })
                    })
                    .collect();
                tags
            }
            None => Ok(vec![]),
            _ => Err(BlogError::ValidationError(format!(
                "Invalid format for field: {}",
                key
            ))),
        }
    }

    /// Validate metadata has required fields
    pub fn validate_metadata(metadata: &BTreeMap<String, Value>) -> Result<()> {
        let required = ["slug", "title", "description", "author", "date"];

        for field in &required {
            if !metadata.contains_key(*field) {
                return Err(BlogError::ValidationError(format!(
                    "Missing required field: {}",
                    field
                )));
            }
        }

        // Validate date format (YYYY-MM-DD)
        let date = Self::get_string(metadata, "date")?;
        let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .map_err(|e| BlogError::FrontmatterError(e.to_string()))?;

        if !date_regex.is_match(&date) {
            return Err(BlogError::ValidationError(
                "Date must be in YYYY-MM-DD format".to_string(),
            ));
        }

        // Validate slug format (lowercase, numbers, hyphens only)
        let slug = Self::get_string(metadata, "slug")?;
        let slug_regex =
            Regex::new(r"^[a-z0-9-]+$").map_err(|e| BlogError::FrontmatterError(e.to_string()))?;

        if !slug_regex.is_match(&slug) {
            return Err(BlogError::ValidationError(
                "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_markdown() -> String {
        r#"---
slug: my-post
title: My Blog Post
description: A test post
author: Test Author
date: 2025-01-15
cover_image: blog/image.jpg
tags:
  - rust
  - webdev
---

# Hello World

This is the content of my blog post."#
            .to_string()
    }

    fn sample_markdown_minimal() -> String {
        r#"---
slug: minimal
title: Minimal Post
description: Minimal description
author: Author
date: 2025-01-15
---

Content only."#
            .to_string()
    }

    #[test]
    fn test_parse_complete_markdown() {
        let parsed = MarkdownParser::parse(&sample_markdown()).unwrap();

        assert_eq!(
            parsed.metadata.get("slug").unwrap().as_str(),
            Some("my-post")
        );
        assert_eq!(
            parsed.metadata.get("title").unwrap().as_str(),
            Some("My Blog Post")
        );
        assert!(parsed.content.contains("# Hello World"));
    }

    #[test]
    fn test_parse_minimal_markdown() {
        let parsed = MarkdownParser::parse(&sample_markdown_minimal()).unwrap();

        assert_eq!(
            parsed.metadata.get("slug").unwrap().as_str(),
            Some("minimal")
        );
        assert!(parsed.content.contains("Content only"));
    }

    #[test]
    fn test_parse_invalid_markdown() {
        let invalid = "This is not valid markdown";
        assert!(MarkdownParser::parse(invalid).is_err());
    }

    #[test]
    fn test_get_string() {
        let parsed = MarkdownParser::parse(&sample_markdown()).unwrap();
        let slug = MarkdownParser::get_string(&parsed.metadata, "slug").unwrap();
        assert_eq!(slug, "my-post");
    }

    #[test]
    fn test_get_optional_string() {
        let parsed = MarkdownParser::parse(&sample_markdown()).unwrap();
        let cover = MarkdownParser::get_optional_string(&parsed.metadata, "cover_image");
        assert_eq!(cover, Some("blog/image.jpg".to_string()));
    }

    #[test]
    fn test_get_string_list() {
        let parsed = MarkdownParser::parse(&sample_markdown()).unwrap();
        let tags = MarkdownParser::get_string_list(&parsed.metadata, "tags").unwrap();
        assert_eq!(tags, vec!["rust", "webdev"]);
    }

    #[test]
    fn test_validate_metadata_valid() {
        let parsed = MarkdownParser::parse(&sample_markdown()).unwrap();
        assert!(MarkdownParser::validate_metadata(&parsed.metadata).is_ok());
    }

    #[test]
    fn test_validate_metadata_invalid_date_format() {
        let markdown = r#"---
slug: test
title: Test
description: Test
author: Author
date: 01-15-2025
---
Content"#;
        let parsed = MarkdownParser::parse(markdown).unwrap();
        assert!(MarkdownParser::validate_metadata(&parsed.metadata).is_err());
    }

    #[test]
    fn test_validate_metadata_invalid_slug() {
        let markdown = r#"---
slug: My-Post
title: Test
description: Test
author: Author
date: 2025-01-15
---
Content"#;
        let parsed = MarkdownParser::parse(markdown).unwrap();
        assert!(MarkdownParser::validate_metadata(&parsed.metadata).is_err());
    }

    #[test]
    fn test_validate_metadata_missing_field() {
        let markdown = r#"---
slug: test
title: Test
description: Test
---
Content"#;
        let parsed = MarkdownParser::parse(markdown).unwrap();
        assert!(MarkdownParser::validate_metadata(&parsed.metadata).is_err());
    }
}
