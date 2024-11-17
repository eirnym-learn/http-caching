use super::http::{HTTPRequest, HTTPResponse};
use std::sync::Arc;

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

/// Generate cache key based on HTTP given request.
pub type CacheKeyFn<AdditionalParams, Headers> =
    Arc<dyn Fn(&HTTPRequest<Headers>, &AdditionalParams) -> CacheRequestKey + Send + Sync>;

/// Deside on data on cache hit.
///
/// Function is called on every cache hit. It's never called on cache miss.
///
pub type CacheKeepFn<AdditionalParams, Headers, CacheTime> = Arc<
    dyn Fn(
            &HTTPRequest<Headers>,
            &HTTPResponse<Headers>,
            &CacheTime,
            &Option<CacheTime>,
            &AdditionalParams,
        ) -> CacheKeepPolicy
        + Send
        + Sync,
>;

/// Return response expiration date on remote data.
///
/// HTTP Response has no body fetched.
///
/// Function is called on every cache miss and cache update (see [`crate::core::cache_config::CacheKeepFn`])
pub type CacheResponsePolicyFn<AdditionalParams, Headers, CacheTime> = Arc<
    dyn Fn(
            &HTTPRequest<Headers>,
            &HTTPResponse<Headers>,
            &AdditionalParams,
        ) -> CacheResponseExpiration<CacheTime>
        + Send
        + Sync,
>;

/// Return current timestamp to be written to cache.
pub type CacheTimeFn<CacheTime> = Arc<dyn Fn() -> CacheTime + Send + Sync>;

/// Additional cache configuration
/// REVIEW: Should it be a trait?
pub struct CacheConfig<AdditionalParams, Headers, CacheTime>
where
    AdditionalParams: Send + Sync,
    CacheTime: Send + Sync,
    Headers: Clone + Send + Sync,
{
    /// Generate cache key based on HTTP given request.
    pub key_fn: CacheKeyFn<AdditionalParams, Headers>,

    /// Return response expiration date on remote data.
    ///
    /// HTTP Response has no body fetched.
    pub cache_keep_fn: CacheKeepFn<AdditionalParams, Headers, CacheTime>,

    /// Return response expiration date on remote data.
    ///
    /// HTTP Response has no body fetched.
    ///
    /// Function is called on every cache miss and cache update (see [`crate::core::cache_config::CacheKeepFn`])
    pub cache_policy_fn: Option<CacheResponsePolicyFn<AdditionalParams, Headers, CacheTime>>,

    /// Return current timestamp to be written as a cache timestamp.
    pub now_fn: CacheTimeFn<CacheTime>,
}
