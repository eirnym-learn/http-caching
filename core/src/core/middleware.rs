use super::cache::{CacheData, CacheManager};
use super::cache_config::{
    self, CacheConfig, CacheKeepFn, CacheKeepPolicy, CacheResponseExpiration,
};
use super::error::Result;
use super::http::{HTTPRequest, HTTPResponse, HttpRequest, HttpResponse};

#[derive(Clone)]
pub enum CacheHitResult {
    /// Cache miss, it hasn't been checked for this request
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
pub trait RequestCaller: Send + Sync {
    type Request: HttpRequest;
    type Response: HttpResponse;
    // TODO: fill extensions as needed
    /// Call remote server to get actual HTTP response
    fn read_remote_headers(
        &self,
        request: &impl HttpRequest,
    ) -> impl std::future::Future<Output = Result<impl HttpResponse>> + Send + Sync;
}

pub trait Middleware: Send + Sync {
    type CacheTimeType: Send + Sync;
    type AdditionalParams: Send + Sync;
    type MiddlewareCacheManager: CacheManager<CacheTimeType = Self::CacheTimeType> + Send + Sync;

    /// Return an instance of cache manager
    fn cache_manager(&self) -> &Self::MiddlewareCacheManager;

    /// Return additional params
    fn additional_params(&self) -> &Self::AdditionalParams;

    /// Return cache config
    fn cache_config(&self) -> &CacheConfig<Self::AdditionalParams, Self::CacheTimeType>;

    /// Handle request and return HTTP response with cache hit result
    ///
    /// if response is None, then request hasn't been made
    fn handle_request(
        &self,
        request: &HTTPRequest,
        remote_caller: &impl RequestCaller,
    ) -> impl std::future::Future<Output = Result<(Option<HTTPResponse>, CacheHitResult)>> + Send
    {
        async {
            let cache_config = self.cache_config();
            let Some(key_fn) = &cache_config.key_fn else {
                return Ok((None, CacheHitResult::CacheOff));
            };

            let additional_params = self.additional_params();

            let cache_config::CacheRequestKey::Key(cache_key) = key_fn(request, additional_params)
            else {
                return Ok((None, CacheHitResult::CacheOff));
            };

            let cache_manager = self.cache_manager();

            // TODO: proper error handling on await
            let cache_data_opt = cache_manager.get(&cache_key).await?;
            let cache_keep = process_cache_hit::<Self::AdditionalParams, Self::CacheTimeType>(
                request,
                &cache_data_opt,
                &additional_params,
                &cache_config.cache_keep_fn,
            );

            match cache_keep {
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
                // no cached data or update
                _ => {}
            }

            // Cache miss
            // TODO: proper error handling on await
            let remote_response = remote_caller.read_remote_headers(request).await?;
            let remote_response_no_body = HTTPResponse::new_no_body(&remote_response);
            let cache_policy = match &cache_config.cache_policy_fn {
                None => CacheResponseExpiration::NoCache,
                Some(cache_policy_fn) => {
                    cache_policy_fn(request, &remote_response_no_body, additional_params)
                }
            };

            let expiration_time = match cache_policy {
                CacheResponseExpiration::NoCache => {
                    return Ok((Some(remote_response_no_body), CacheHitResult::CacheOff))
                }
                CacheResponseExpiration::CacheWithoutExpirationDate => None,
                CacheResponseExpiration::CacheWithExpirationDate(expiration_date) => {
                    Some(expiration_date)
                }
            };

            // TODO: proper error handling on await
            let remote_response_with_body = HTTPResponse::new(&remote_response).await?;
            let new_cache_data = CacheData::<Self::CacheTimeType> {
                call_timestamp: (cache_config.now_fn)(),
                expiration_time,
                http_request: request.clone(),
                http_response: remote_response_with_body.clone(),
            };

            // TODO: proper error handling on await
            cache_manager.put(&cache_key, &new_cache_data).await?;

            let cache_hit_result = if matches!(cache_keep, Some(CacheKeepPolicy::Update)) {
                CacheHitResult::CacheUpdate
            } else {
                CacheHitResult::CacheMiss
            };

            return Ok((Some(remote_response_with_body), cache_hit_result));
        }
    }
}

fn process_cache_hit<AdditionalParams, CacheTimeType>(
    request: &HTTPRequest,
    cache_data_opt: &Option<CacheData<CacheTimeType>>,
    additional_params: &AdditionalParams,
    cache_keep_fn: &Option<CacheKeepFn<AdditionalParams, CacheTimeType>>,
) -> Option<CacheKeepPolicy> {
    let Some(cache_data) = cache_data_opt else {
        return None;
    };

    Some(match cache_keep_fn {
        None => CacheKeepPolicy::Keep,
        Some(cache_keep_fn) => cache_keep_fn(
            request,
            &cache_data.http_response,
            &cache_data.expiration_time,
            &additional_params,
        ),
    })
}
