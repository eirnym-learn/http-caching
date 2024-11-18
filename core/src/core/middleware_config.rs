use super::error::Result;
use super::http::{HTTPRequest, HTTPResponse, HttpResponse};

/// Represent generated cache key based on the request.
pub enum CacheRequestKey {
    /// No key associated with a request.
    NoKey,
    /// Given key associated with the request.
    Key(String),
}

/// If cached data should be kept, updated or evicted.
pub enum CacheKeepPolicy {
    /// Return remote data without touching cached data.
    Skip,

    /// Keep data in cache and return cached content.
    Keep,

    /// Return remote data and update it in cache later.
    ///
    /// Note: response might be rejected later to be written into cache.
    /// In this case, cache data will be preserved as is.
    Update,

    /// Return remote data and evict data from cache.
    Evict,
}

/// Expiration dates for cache
#[derive(Debug, PartialEq)]
pub enum CacheResponseExpiration<CacheTime> {
    /// Don't cache response
    NoCache,

    /// Cache response with indefinite expiration date.
    CacheWithoutExpirationDate,

    /// Cache response with given expiration date.
    CacheWithExpirationDate(CacheTime),
}

/// Configure middleware caching policies.
pub trait MiddlewareCachingConfig {
    type Headers: Clone + Send + Sync;
    type CacheTime: Send + Sync;

    /// Generate cache key based on HTTP given request.
    fn key(&self, request: &HTTPRequest<Self::Headers>) -> CacheRequestKey;

    /// Deside how cached data should be kept.
    ///
    /// Function is called on every cache hit. It's never called on cache miss.
    fn cache_keep(
        &self,
        request: &HTTPRequest<Self::Headers>,
        response: &HTTPResponse<Self::Headers>,
        call_timestamp: &Self::CacheTime,
        expiration_time: &Option<Self::CacheTime>,
    ) -> CacheKeepPolicy;

    /// Return response expiration date on remote data.
    ///
    /// HTTP Response has no body fetched.
    ///
    /// Function is called on every cache miss and cache update.
    fn cache_response(
        &self,
        request: &HTTPRequest<Self::Headers>,
        response: &HTTPResponse<Self::Headers>,
    ) -> Option<CacheResponseExpiration<Self::CacheTime>>;
}

/// Abstraction to do remote call for given request.
pub trait RequestCaller: Send + Sync {
    type Headers: Clone + Send + Sync;
    type Response: HttpResponse<Headers = Self::Headers>;

    /// Call remote server to get actual HTTP response
    fn read_remote_headers(
        &self,
        request: &HTTPRequest<Self::Headers>,
    ) -> impl std::future::Future<Output = Result<Self::Response>> + Send + Sync;
}
