use crate::error::{BlogError, Result};
use crate::parser::MarkdownParser;
use crate::types::{BlogMeta, BlogPost};
use cloudflare_kv::client::KvClient;
use std::path::Path;
use tracing::debug;

const BLOG_LIST_KEY: &str = "_blog_list";
const POST_KEY_PREFIX: &str = "post:";

/// Blog post publisher for managing blog posts in Cloudflare KV
pub struct BlogPublisher<'a> {
    client: &'a KvClient,
}

impl<'a> BlogPublisher<'a> {
    /// Create a new blog publisher
    pub fn new(client: &'a KvClient) -> Self {
        Self { client }
    }

    /// Publish a blog post from a markdown file
    pub async fn publish_from_file(&self, file_path: &Path) -> Result<()> {
        debug!("Publishing blog post from: {}", file_path.display());

        // Read file
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| BlogError::IoError(e))?;

        // Parse markdown
        let parsed = MarkdownParser::parse(&content)?;

        // Validate metadata
        MarkdownParser::validate_metadata(&parsed.metadata)?;

        // Extract metadata
        let slug = MarkdownParser::get_string(&parsed.metadata, "slug")?;
        let title = MarkdownParser::get_string(&parsed.metadata, "title")?;
        let description = MarkdownParser::get_string(&parsed.metadata, "description")?;
        let author = MarkdownParser::get_string(&parsed.metadata, "author")?;
        let date = MarkdownParser::get_string(&parsed.metadata, "date")?;
        let cover_image = MarkdownParser::get_optional_string(&parsed.metadata, "cover_image");
        let tags = MarkdownParser::get_string_list(&parsed.metadata, "tags")?;

        // Create blog post
        let post = BlogPost {
            slug: slug.clone(),
            title: title.clone(),
            description: description.clone(),
            author: author.clone(),
            date: date.clone(),
            cover_image: cover_image.clone(),
            tags: tags.clone(),
            content: parsed.content.clone(),
        };

        // Save post to KV
        self.save_post(&post).await?;

        // Update blog list
        self.update_blog_list(&post.meta()).await?;

        debug!("Successfully published: {}", title);
        Ok(())
    }

    /// Save a blog post to KV
    async fn save_post(&self, post: &BlogPost) -> Result<()> {
        let key = format!("{}{}", POST_KEY_PREFIX, post.slug);
        let value = serde_json::to_string(post)
            .map_err(BlogError::JsonError)?;

        self.client
            .put(&key, value.as_bytes())
            .await
            .map_err(|e| BlogError::KvError(e.to_string()))?;

        debug!("Saved post content for: {}", post.slug);
        Ok(())
    }

    /// Get a blog post by slug
    pub async fn get_post(&self, slug: &str) -> Result<Option<BlogPost>> {
        let key = format!("{}{}", POST_KEY_PREFIX, slug);

        match self.client.get(&key).await {
            Ok(Some(kv_pair)) => {
                let post: BlogPost = serde_json::from_str(&kv_pair.value)
                    .map_err(BlogError::JsonError)?;
                Ok(Some(post))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BlogError::KvError(e.to_string())),
        }
    }

    /// Delete a blog post by slug
    pub async fn delete_post(&self, slug: &str) -> Result<()> {
        let key = format!("{}{}", POST_KEY_PREFIX, slug);

        // Delete the post
        self.client
            .delete(&key)
            .await
            .map_err(|e| BlogError::KvError(e.to_string()))?;

        debug!("Deleted post content for: {}", slug);

        // Remove from blog list
        self.remove_from_blog_list(slug).await?;

        Ok(())
    }

    /// Get all blog posts (metadata only)
    pub async fn list_posts(&self) -> Result<Vec<BlogMeta>> {
        match self.get_blog_list().await {
            Ok(posts) => Ok(posts),
            Err(e) => {
                if e.to_string().contains("not found") {
                    Ok(vec![])
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get the blog list from KV
    async fn get_blog_list(&self) -> Result<Vec<BlogMeta>> {
        match self.client.get(BLOG_LIST_KEY).await {
            Ok(Some(kv_pair)) => {
                let posts: Vec<BlogMeta> = serde_json::from_str(&kv_pair.value)
                    .map_err(BlogError::JsonError)?;
                Ok(posts)
            }
            Ok(None) => Ok(vec![]),
            Err(e) => Err(BlogError::KvError(e.to_string())),
        }
    }

    /// Update the blog list after publishing a post
    async fn update_blog_list(&self, post_meta: &BlogMeta) -> Result<()> {
        let mut blog_list = self.get_blog_list().await?;

        // Check if post already exists
        if let Some(pos) = blog_list.iter().position(|p| p.slug == post_meta.slug) {
            blog_list[pos] = post_meta.clone();
            debug!("Updated existing entry in blog list");
        } else {
            blog_list.insert(0, post_meta.clone()); // Insert at beginning (newest first)
            debug!("Added new entry to blog list");
        }

        // Sort by date (newest first)
        blog_list.sort_by(|a, b| b.date.cmp(&a.date));

        // Save updated list
        let list_json = serde_json::to_string(&blog_list)
            .map_err(BlogError::JsonError)?;

        self.client
            .put(BLOG_LIST_KEY, list_json.as_bytes())
            .await
            .map_err(|e| BlogError::KvError(e.to_string()))?;

        debug!("Updated blog list ({} posts)", blog_list.len());
        Ok(())
    }

    /// Remove a post from the blog list
    async fn remove_from_blog_list(&self, slug: &str) -> Result<()> {
        let mut blog_list = self.get_blog_list().await?;
        let original_len = blog_list.len();

        blog_list.retain(|p| p.slug != slug);

        if blog_list.len() < original_len {
            let list_json = serde_json::to_string(&blog_list)
                .map_err(BlogError::JsonError)?;

            self.client
                .put(BLOG_LIST_KEY, list_json.as_bytes())
                .await
                .map_err(|e| BlogError::KvError(e.to_string()))?;

            debug!("Removed post from blog list");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudflare_kv::types::AuthCredentials;

    fn create_test_client() -> KvClient {
        let creds = AuthCredentials::token("test-token");
        let config = cloudflare_kv::ClientConfig::new(
            "test-account",
            "test-namespace",
            creds,
        );
        KvClient::new(config)
    }

    #[test]
    fn test_publisher_creation() {
        let client = create_test_client();
        let _publisher = BlogPublisher::new(&client);
        // Publisher created successfully
    }

    #[test]
    fn test_blog_list_key_constant() {
        assert_eq!(BLOG_LIST_KEY, "_blog_list");
    }

    #[test]
    fn test_post_key_prefix_constant() {
        assert_eq!(POST_KEY_PREFIX, "post:");
    }

    #[test]
    fn test_post_key_format() {
        let slug = "my-post";
        let key = format!("{}{}", POST_KEY_PREFIX, slug);
        assert_eq!(key, "post:my-post");
    }
}
