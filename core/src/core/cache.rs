use super::{
    error::Result,
    http::{HTTPRequest, HTTPResponse},
};

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
pub trait CacheManager: Send + Sync {
    /// Attempt to pull a cached response.
    fn get(
        &self,
        cache_key: &String,
    ) -> impl std::future::Future<Output = Result<Option<CacheData>>> + Send + Sync;

    /// Attempt to put data in cache.
    fn put(
        &self,
        cache_key: &String,
        data: &CacheData,
    ) -> impl std::future::Future<Output = Result<()>> + Send + Sync;

    /// Attempt to remove a record from cache.
    fn delete(
        &self,
        cache_key: &String,
    ) -> impl std::future::Future<Output = Result<Option<CacheData>>> + Send + Sync;
}
