use super::cache::CacheTimestamp;
use super::http::{HTTPRequest, HTTPResponse};
use std::sync::Arc;

/// Policy how response needs to be cached
pub enum CacheResponsePolicy {
    /// Don't cache response
    NoCache,

    /// Cache without any expiration date. In another words, expiration date is in the infinite future.
    CacheWithoutExpirationDate,

    /// Cache with given expiration date.
    CacheWithExpirationDate(CacheTimestamp),
}

/// Represent generated cache key.
pub enum CacheKey {
    /// No key associated with a request.
    NoKey,
    /// Given key associated with the request.
    Key(String),
}

/// If element should be kept or evicted.
pub enum CacheKeep {
    /// Data should be kept.
    Keep,

    /// Data should be updated.
    Update,

    /// Data should be evicted from cache.
    Evict,
}

/// Generate key based on HTTP given request.
///
/// An override can be useful to override key function to exclude certain query parameters, fragments, etc,
/// or use additional headers or even another hash function than default one.
///
/// A closure takes something implementing [`crate::http::HttpRequest`] and returns a [`CacheKey`].
pub type CacheKeyFn<AdditionalParams> =
    Arc<dyn Fn(&HTTPRequest, &AdditionalParams) -> CacheKey + Send + Sync>;

/// Return what should be done with given cache entry.
///
/// E.g. cache entry should be kept, updated or evicted.
/// If cache entry should be updated or evicted, new actual request would run.
///
/// CacheTimestamp parameter is an expiration date associated with given cache entry.
///
/// NOTE: Function is called for every request, only on cache hit.
///
/// NOTE: If closure call results in real request to be made, cache policy would be called.
/// If NoCache policy is associated with HTTP response, cache entry will be evicted.
///
/// A closure that takes [`crate::http::HTTPRequest`], [`crate::http::HTTPResponse`] and expiration timestamp from cache
/// and returns a [`CacheBust`].
pub type CacheKeepFn<AdditionalParams> = Arc<
    dyn Fn(&HTTPRequest, &HTTPResponse, &Option<CacheTimestamp>, &AdditionalParams) -> CacheKeep
        + Send
        + Sync,
>;

/// Return cache policy how given real response data needs to be cached.
///
/// Provided HTTP Response to this closure has no body read from the network.
///
/// NOTE: Function is called on every request, only on cache miss.
///
/// A closure that takes [`crate::http::HTTPRequest`] and [`crate::http::HttpResponse`] headers and returns a [`CachePolicy`].
pub type CacheResponsePolicyFn<AdditionalParams> = Arc<
    dyn Fn(&HTTPRequest, &HTTPResponse, &AdditionalParams) -> CacheResponsePolicy + Send + Sync,
>;

/// Additional cache configuration
pub struct CacheConfig<AdditionalParams> {
    /// Generate key based on HTTP given request. CacheKey::NoKey by default.
    ///
    /// An override can be useful to override key function to exclude certain query parameters, fragments, etc,
    /// or use additional headers or even another hash function than default one.
    pub key_fn: Option<CacheKeyFn<AdditionalParams>>,

    /// Return if cached entry should be kept or not. CacheKeep::Keep by default.
    ///
    /// E.g. cache entry should be kept, updated or evicted.
    /// If cache entry should be updated or evicted, new actual request would run.
    ///
    /// CacheTimestamp parameter is an expiration date associated with given cache entry.
    ///
    /// NOTE: Function is called for every request, only on cache hit.
    ///
    /// NOTE: If closure call results in real request to be made, cache policy would be called.
    /// If NoCache policy is associated with HTTP response, cache entry will be evicted.
    pub cache_keep_fn: Option<CacheKeepFn<AdditionalParams>>,

    /// Return cache policy how given real response data needs to be cached.
    ///
    /// Provided HTTP Response to this closure has no body read from the network.
    ///
    /// NOTE: Function is called on every request, only on cache miss.
    pub cache_policy_fn: Option<CacheResponsePolicyFn<AdditionalParams>>,
}
