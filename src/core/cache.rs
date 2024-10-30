use super::{
    error::Result,
    http::{HTTPRequest, HTTPResponse},
};
use std::borrow::Cow;

pub type CacheTimestamp = chrono::DateTime<chrono::Utc>;

/// Data to be stored in cache
pub struct CacheData {
    /// Timestamp when call has been recorded
    pub call_timestamp: CacheTimestamp,

    /// Presumable time to expire this record. None means to cache indefinitely
    pub expiration_time: Option<CacheTimestamp>,

    /// HTTP Request data.
    /// In some cases it could be useful to get it back.
    pub http_request: HTTPRequest,

    /// HTTP Response data
    pub http_response: HTTPResponse,
}

/// A trait providing methods for storing, reading, and removing cache records.
// #[async_trait::async_trait]
pub trait CacheManager: Send + Sync + 'static {
    /// Attempt to pull a cached response.
    async fn get(&self, cache_key: &String) -> Result<Option<CacheData>>;

    /// Attempt to put data in cache.
    async fn put(&self, cache_key: &String, data: &CacheData) -> Result<()>;

    /// Attempt to remove a record from cache.
    async fn delete(&self, cache_key: &String) -> Result<Option<CacheData>>;
}
