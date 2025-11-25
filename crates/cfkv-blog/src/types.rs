use serde::{Deserialize, Serialize};

/// Blog post metadata (for the blog list)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlogMeta {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub date: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
}

/// Complete blog post (with content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub date: String,
    pub cover_image: Option<String>,
    pub tags: Vec<String>,
    pub content: String,
}

impl BlogPost {
    /// Extract metadata from the blog post
    pub fn meta(&self) -> BlogMeta {
        BlogMeta {
            slug: self.slug.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            date: self.date.clone(),
            cover_image: self.cover_image.clone(),
            tags: self.tags.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blog_meta_creation() {
        let meta = BlogMeta {
            slug: "test-post".to_string(),
            title: "Test Post".to_string(),
            description: "A test post".to_string(),
            author: "Author".to_string(),
            date: "2025-01-15".to_string(),
            cover_image: Some("image.jpg".to_string()),
            tags: vec!["rust".to_string(), "web".to_string()],
        };

        assert_eq!(meta.slug, "test-post");
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn test_blog_post_meta_extraction() {
        let post = BlogPost {
            slug: "my-post".to_string(),
            title: "My Post".to_string(),
            description: "Description".to_string(),
            author: "Author".to_string(),
            date: "2025-01-15".to_string(),
            cover_image: None,
            tags: vec!["test".to_string()],
            content: "# Content".to_string(),
        };

        let meta = post.meta();
        assert_eq!(meta.slug, "my-post");
        assert_eq!(meta.title, "My Post");
        assert!(meta.cover_image.is_none());
    }

    #[test]
    fn test_blog_meta_equality() {
        let meta1 = BlogMeta {
            slug: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            date: "2025-01-15".to_string(),
            cover_image: None,
            tags: vec![],
        };

        let meta2 = meta1.clone();
        assert_eq!(meta1, meta2);
    }
}
