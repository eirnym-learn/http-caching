use super::cache::{CacheData, CacheManager};
use super::error::Result;
use super::http::{HTTPRequest, HTTPResponse, HttpResponse};
use super::middleware_config::{self, CacheKeepPolicy, CacheResponseExpiration, MiddlewareConfig};

#[derive(Clone)]
pub enum CacheHitResult {
    /// Cache miss, cache hasn't been checked for this request (keep policy skip was returned)
    CacheOff,
    /// Cache miss, new data from remote has been stored
    CacheMiss,
    /// Cache hit, data has been taken from cache
    CacheHit,
    /// Cache hit, data has been updated from remote
    CacheUpdate,
    /// Cache hit, cached data has been evicted, data has been retrieved from remote
    CacheEvict,
}

/// Abstraction to do remote call for given request
// TODO: fill extensions as needed
pub trait RequestCaller: Send + Sync {
    type Headers: Clone + Send + Sync;
    type Response: HttpResponse<Headers = Self::Headers>;
    /// Call remote server to get actual HTTP response
    fn read_remote_headers(
        &self,
        request: &HTTPRequest<Self::Headers>,
    ) -> impl std::future::Future<Output = Result<Self::Response>> + Send + Sync;
}

// TODO: Rewrite trait to a function with parameters.
// TODO: Move parameters into a trait/structure.
pub trait Middleware: Send + Sync {
    type Headers: Clone + Send + Sync;
    type CacheTime: Send + Sync;
    type AdditionalParams: Send + Sync;
    type MiddlewareCacheManager: CacheManager<CacheTime = Self::CacheTime, Headers = Self::Headers>
        + Send
        + Sync;

    /// Return an instance of cache manager
    fn cache_manager(&self) -> &Self::MiddlewareCacheManager;

    /// Return additional params
    fn additional_params(&self) -> &Self::AdditionalParams;

    /// Return cache config
    fn middeware_config(
        &self,
    ) -> &MiddlewareConfig<Self::AdditionalParams, Self::Headers, Self::CacheTime>;

    /// Handle request and return HTTP response with cache hit result
    ///
    /// if response is None, then request hasn't been made, and caller should do it manually
    fn handle_request(
        &self,
        request: &HTTPRequest<Self::Headers>,
        remote_caller: &impl RequestCaller<Headers = Self::Headers>,
    ) -> impl std::future::Future<
        Output = Result<(Option<HTTPResponse<Self::Headers>>, CacheHitResult)>,
    > + Send {
        async {
            let cache_config = self.middeware_config();
            let additional_params = self.additional_params();

            let middleware_config::CacheRequestKey::Key(cache_key) =
                (cache_config.key_fn)(request, additional_params)
            else {
                return Ok((None, CacheHitResult::CacheOff));
            };

            let cache_manager = self.cache_manager();

            // TODO: proper error handling on await
            let cache_data_opt = cache_manager.get(&cache_key).await?;

            let cache_keep = cache_data_opt.as_ref().map(|cache_data| {
                (cache_config.cache_keep_fn)(
                    request,
                    &cache_data.http_response,
                    &cache_data.call_timestamp,
                    &cache_data.expiration_time,
                    additional_params,
                )
            });

            match cache_keep {
                Some(CacheKeepPolicy::Skip) => {
                    return Ok((None, CacheHitResult::CacheOff));
                }
                Some(CacheKeepPolicy::Keep) => {
                    return Ok((
                        Some(cache_data_opt.unwrap().http_response),
                        CacheHitResult::CacheHit,
                    ))
                }
                Some(CacheKeepPolicy::Evict) => {
                    // TODO: proper error handling on await
                    cache_manager.delete(&cache_key).await?;
                    return Ok((None, CacheHitResult::CacheEvict));
                }
                // cache data needs to be updated
                Some(CacheKeepPolicy::Update) => {}
                // cache miss => deside later
                None => {}
            }

            // Cache miss
            // TODO: proper error handling on await
            let remote_response = remote_caller.read_remote_headers(request).await?;

            let remote_response_no_body = HTTPResponse {
                version: remote_response.version(),
                status: remote_response.status(),
                reason: remote_response.reason().clone(),
                url: remote_response.url().clone(),
                headers: remote_response.headers().clone(),
                body: vec![],
            };

            let cache_policy = match &cache_config.cache_policy_fn {
                None => CacheResponseExpiration::NoCache,
                Some(cache_policy_fn) => {
                    cache_policy_fn(request, &remote_response_no_body, additional_params)
                }
            };

            // TODO: proper error handling on await
            // Copy already read data and append body.
            let remote_response_with_body = HTTPResponse {
                body: remote_response.body().await?,
                version: remote_response_no_body.version,
                url: remote_response_no_body.url,
                status: remote_response_no_body.status,
                reason: remote_response_no_body.reason,
                headers: remote_response_no_body.headers,
            };

            let expiration_time = match cache_policy {
                CacheResponseExpiration::NoCache => {
                    return Ok((Some(remote_response_with_body), CacheHitResult::CacheOff));
                }
                CacheResponseExpiration::CacheWithoutExpirationDate => None,
                CacheResponseExpiration::CacheWithExpirationDate(expiration_date) => {
                    Some(expiration_date)
                }
            };

            let new_cache_data = CacheData::<Self::CacheTime, Self::Headers> {
                call_timestamp: (cache_config.time_now_fn)(),
                expiration_time,
                http_request: HTTPRequest::new(request),
                http_response: remote_response_with_body.clone(),
            };

            // TODO: proper error handling on await
            cache_manager.put(&cache_key, &new_cache_data).await?;

            let cache_hit_result = if matches!(cache_keep, Some(CacheKeepPolicy::Update)) {
                CacheHitResult::CacheUpdate
            } else {
                CacheHitResult::CacheMiss
            };

            Ok((Some(remote_response_with_body), cache_hit_result))
        }
    }
}
