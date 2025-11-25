use crate::error::Result;
use crate::KvClient;

/// Batch operation builder for efficient bulk operations
pub struct BatchBuilder {
    operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug)]
pub enum BatchOperation {
    Put { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl BatchBuilder {
    /// Create a new batch builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add a put operation
    pub fn put(mut self, key: impl Into<String>, value: impl AsRef<[u8]>) -> Self {
        self.operations.push(BatchOperation::Put {
            key: key.into(),
            value: value.as_ref().to_vec(),
        });
        self
    }

    /// Add a delete operation
    pub fn delete(mut self, key: impl Into<String>) -> Self {
        self.operations.push(BatchOperation::Delete {
            key: key.into(),
        });
        self
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get all operations
    pub fn operations(&self) -> &[BatchOperation] {
        &self.operations
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Paginated iterator for efficient list operations
pub struct PaginatedIterator {
    client: std::sync::Arc<KvClient>,
    current_cursor: Option<String>,
    limit: u32,
    exhausted: bool,
}

impl PaginatedIterator {
    /// Create a new paginated iterator
    pub fn new(client: std::sync::Arc<KvClient>, limit: u32) -> Self {
        Self {
            client,
            current_cursor: None,
            limit,
            exhausted: false,
        }
    }

    /// Get the next page of results
    pub async fn next_page(&mut self) -> Result<Option<Vec<String>>> {
        if self.exhausted {
            return Ok(None);
        }

        let response = self
            .client
            .list(Some(
                crate::types::PaginationParams::new()
                    .with_limit(self.limit)
                    .with_cursor(self.current_cursor.clone().unwrap_or_default()),
            ))
            .await?;

        if response.keys.is_empty() && self.current_cursor.is_none() {
            return Ok(None);
        }

        self.exhausted = response.list_complete;
        self.current_cursor = response.cursor;

        let keys: Vec<String> = response.keys.into_iter().map(|k| k.name).collect();

        if keys.is_empty() {
            Ok(None)
        } else {
            Ok(Some(keys))
        }
    }

    /// Check if there are more pages
    pub fn has_more(&self) -> bool {
        !self.exhausted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_builder_operations() {
        let batch = BatchBuilder::new()
            .put("key1", "value1")
            .put("key2", "value2")
            .delete("key3");

        assert_eq!(batch.len(), 3);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_builder_empty() {
        let batch = BatchBuilder::default();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_batch_builder_single_op() {
        assert_eq!(BatchBuilder::new().put("key", "value").len(), 1);
        assert_eq!(BatchBuilder::new().delete("key").len(), 1);
    }

    #[test]
    fn test_batch_builder_large() {
        let mut batch = BatchBuilder::new();
        for i in 0..100 {
            batch = batch.put(format!("key-{}", i), format!("value-{}", i));
        }
        assert_eq!(batch.len(), 100);
    }

    #[test]
    fn test_batch_operations_access() {
        let batch = BatchBuilder::new().put("a", "1").delete("b").put("c", "3");
        let ops = batch.operations();
        assert_eq!(ops.len(), 3);
    }
}
