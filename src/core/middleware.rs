use super::cache::{CacheData, CacheManager};
use super::cache_config::{self, CacheConfig, CacheKeep, CacheKeepFn, CacheResponsePolicy};
use super::error::Result;
use super::http::{HTTPRequest, HTTPResponse};

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
pub trait RequestCaller: Send + Sync + 'static {
    /// Call remote server to get actual HTTP response
    async fn read_remote_headers(&self, request: &HTTPRequest) -> Result<HTTPResponse>;
}

pub trait Middleware: Send + Sync + 'static {
    type AdditionalParams;
    type MiddlewareCacheManager: CacheManager;

    /// Return an instance of cache manager
    fn cache_manager(&self) -> &Self::MiddlewareCacheManager;

    /// Return additional params
    fn additional_params(&self) -> &Self::AdditionalParams;

    /// Return cache config
    fn cache_config(&self) -> &CacheConfig<Self::AdditionalParams>;

    /// Handle request and return HTTP response with cache hit result
    ///
    /// if response is None, then request hasn't been made
    async fn handle_request(
        &self,
        request: &HTTPRequest,
        remote_caller: &impl RequestCaller,
    ) -> Result<(Option<HTTPResponse>, CacheHitResult)> {
        let cache_config = self.cache_config();
        let Some(key_fn) = &cache_config.key_fn else {
            return Ok((None, CacheHitResult::CacheOff));
        };

        let additional_params = self.additional_params();

        let cache_config::CacheKey::Key(cache_key) = key_fn(request, additional_params) else {
            return Ok((None, CacheHitResult::CacheOff));
        };

        let cache_manager = self.cache_manager();

        // TODO: proper error handling on await
        let cache_data_opt = cache_manager.get(&cache_key).await?;
        let cache_keep = process_cache_hit::<Self::AdditionalParams>(
            request,
            &cache_data_opt,
            &additional_params,
            &cache_config.cache_keep_fn,
        );

        match cache_keep {
            Some(CacheKeep::Keep) => {
                return Ok((
                    Some(cache_data_opt.unwrap().http_response),
                    CacheHitResult::CacheHit,
                ))
            }
            Some(CacheKeep::Evict) => {
                cache_manager.delete(&cache_key);
                return Ok((None, CacheHitResult::CacheEvict));
            }
            // no cached data or update
            _ => {}
        }

        // Cache miss
        // TODO: proper error handling on await
        let remote_response = remote_caller.read_remote_headers(request).await?;
        let cache_policy = match &cache_config.cache_policy_fn {
            None => CacheResponsePolicy::NoCache,
            Some(cache_policy_fn) => cache_policy_fn(request, &remote_response, additional_params),
        };

        let expiration_time = match cache_policy {
            CacheResponsePolicy::NoCache => {
                return Ok((Some(remote_response), CacheHitResult::CacheOff))
            }
            CacheResponsePolicy::CacheWithoutExpirationDate => None,
            CacheResponsePolicy::CacheWithExpirationDate(expiration_date) => Some(expiration_date),
        };
        let new_cache_data = CacheData {
            call_timestamp: chrono::offset::Utc::now(),
            expiration_time,
            http_request: request.clone(),
            http_response: remote_response.clone(),
        };
        cache_manager.put(&cache_key, &new_cache_data);

        let cache_hit_result = if matches!(cache_keep, Some(CacheKeep::Update)) {
            CacheHitResult::CacheUpdate
        } else {
            CacheHitResult::CacheMiss
        };
        return Ok((Some(remote_response), cache_hit_result));
    }
}

fn process_cache_hit<AdditionalParams>(
    request: &HTTPRequest,
    cache_data_opt: &Option<CacheData>,
    additional_params: &AdditionalParams,
    cache_keep_fn: &Option<CacheKeepFn<AdditionalParams>>,
) -> Option<CacheKeep> {
    let Some(cache_data) = cache_data_opt else {
        return None;
    };

    Some(match cache_keep_fn {
        None => CacheKeep::Keep,
        Some(cache_keep_fn) => cache_keep_fn(
            request,
            &cache_data.http_response,
            &cache_data.expiration_time,
            &additional_params,
        ),
    })
}
