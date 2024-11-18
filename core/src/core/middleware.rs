use std::sync::Arc;

use super::cache::{CacheData, CacheManager};
use super::error::Result;
use super::http::{HTTPRequest, HTTPResponse, HttpResponse};
use super::middleware_config::{CacheKeepPolicy, CacheResponseExpiration};
use super::{middleware_config, Error};

/// Return current timestamp to be written to cache.
pub type CurrentTimeFn<CacheTime> = Arc<dyn Fn() -> CacheTime + Send + Sync>;

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

pub async fn handle_response_caching<
    'src,
    Headers,
    CacheTime,
    MiddlewareCacheManager,
    MiddlewareCachingConfig,
    RequestCaller,
>(
    request: &'src HTTPRequest<Headers>,
    request_caller: &'src RequestCaller,
    cache_manager: &'src MiddlewareCacheManager,
    middleware_caching_config: &'src MiddlewareCachingConfig,
    current_time_fn: &'src CurrentTimeFn<CacheTime>,
) -> Result<(Option<HTTPResponse<Headers>>, CacheHitResult)>
where
    CacheTime: Send + Sync,
    Headers: Clone + Send + Sync,
    MiddlewareCacheManager: CacheManager<Headers = Headers, CacheTime = CacheTime> + Send + Sync,
    MiddlewareCachingConfig: middleware_config::MiddlewareCachingConfig<Headers = Headers, CacheTime = CacheTime>
        + Send
        + Sync,
    RequestCaller: middleware_config::RequestCaller<Headers = Headers>,
{
    let middleware_config::CacheRequestKey::Key(cache_key) = middleware_caching_config.key(request)
    else {
        return Ok((None, CacheHitResult::CacheOff));
    };

    // TODO: proper error handling on await
    let cache_data_opt = cache_manager.get(&cache_key).await?;

    let cache_keep = cache_data_opt.as_ref().map(|cache_data| {
        middleware_caching_config.cache_keep(
            request,
            &cache_data.http_response,
            &cache_data.call_timestamp,
            &cache_data.expiration_time,
        )
    });

    match cache_keep {
        Some(CacheKeepPolicy::Skip) => {
            return Ok((None, CacheHitResult::CacheOff));
        }
        Some(CacheKeepPolicy::Keep) => {
            let Some(cached_data) = cache_data_opt else {
                return Err(Error::FIXME);
            };
            return Ok((Some(cached_data.http_response), CacheHitResult::CacheHit));
        }
        Some(CacheKeepPolicy::Evict) => {
            // TODO: proper error handling on await
            cache_manager.delete(&cache_key).await?;
            return Ok((None, CacheHitResult::CacheEvict));
        }
        // cache data needs to be updated or there's a cache miss => process later
        Some(CacheKeepPolicy::Update) | None => {}
    }

    // Cache miss
    // TODO: proper error handling on await
    let remote_response = request_caller.read_remote_headers(request).await?;

    let remote_response_no_body = HTTPResponse {
        version: remote_response.version(),
        status: remote_response.status(),
        reason: remote_response.reason(),
        url: remote_response.url(),
        headers: remote_response.headers().clone(),
        body: vec![],
    };

    let cache_policy = middleware_caching_config
        .cache_response(request, &remote_response_no_body)
        .unwrap_or(CacheResponseExpiration::<CacheTime>::NoCache);

    // REVIEW: don't read whole body if NoCache returned?

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
        CacheResponseExpiration::CacheWithExpirationDate(expiration_date) => Some(expiration_date),
    };
    let call_timestamp = current_time_fn();
    let new_cache_data = CacheData::<Headers, CacheTime> {
        call_timestamp,
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
