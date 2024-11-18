use std::sync::Arc;

use crate::core::{
    http::{HTTPRequest, HTTPResponse},
    middleware_config::{
        CacheKeepPolicy, CacheRequestKey, CacheResponseExpiration, MiddlewareCachingConfig,
    },
};

/// Generate cache key based on HTTP given request.
pub type CacheKeyFn<AdditionalParams, Headers> =
    Arc<dyn Fn(&HTTPRequest<Headers>, &AdditionalParams) -> CacheRequestKey + Send + Sync>;

/// Deside how cached data should be kept.
///
/// Function is called on every cache hit. It's never called on cache miss.
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
/// Function is called on every cache miss and cache update.
pub type CacheResponsePolicyFn<AdditionalParams, Headers, CacheTime> = Arc<
    dyn Fn(
            &HTTPRequest<Headers>,
            &HTTPResponse<Headers>,
            &AdditionalParams,
        ) -> CacheResponseExpiration<CacheTime>
        + Send
        + Sync,
>;

/// Simple cache configuration with additional params
pub struct SimpleMiddlewareCachingConfig<AdditionalParams, Headers, CacheTime>
where
    AdditionalParams: Send + Sync,
    Headers: Clone + Send + Sync,
    CacheTime: Send + Sync,
{
    /// Addtitional parameters to pass to functions below
    pub additional_parameters: AdditionalParams,

    /// Generate cache key based on HTTP given request.
    pub key_fn: CacheKeyFn<AdditionalParams, Headers>,

    /// Deside how cached data should be kept.
    ///
    /// Function is called on every cache hit. It's never called on cache miss.
    pub cache_keep_fn: CacheKeepFn<AdditionalParams, Headers, CacheTime>,

    /// Return response expiration date on remote data.
    ///
    /// HTTP Response has no body fetched.
    ///
    /// Function is called on every cache miss and cache update.
    pub cache_policy_fn: Option<CacheResponsePolicyFn<AdditionalParams, Headers, CacheTime>>,
}

impl<AdditionalParams, Headers, CacheTime> MiddlewareCachingConfig
    for SimpleMiddlewareCachingConfig<AdditionalParams, Headers, CacheTime>
where
    AdditionalParams: Send + Sync,
    Headers: Clone + Send + Sync,
    CacheTime: Send + Sync,
{
    type Headers = Headers;
    type CacheTime = CacheTime;

    #[inline]
    fn key(&self, request: &HTTPRequest<Headers>) -> CacheRequestKey {
        (self.key_fn)(request, &self.additional_parameters)
    }

    #[inline]
    fn cache_keep(
        &self,
        request: &HTTPRequest<Headers>,
        response: &HTTPResponse<Headers>,
        call_timestamp: &CacheTime,
        expiration_time: &Option<CacheTime>,
    ) -> CacheKeepPolicy {
        (self.cache_keep_fn)(
            request,
            response,
            call_timestamp,
            expiration_time,
            &self.additional_parameters,
        )
    }

    #[inline]
    fn cache_response(
        &self,
        request: &HTTPRequest<Headers>,
        response: &HTTPResponse<Headers>,
    ) -> Option<CacheResponseExpiration<CacheTime>> {
        self.cache_policy_fn
            .as_ref()
            .map(|fun| fun(request, response, &self.additional_parameters))
    }
}
