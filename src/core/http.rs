use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Http Request status
/// async?
pub trait HttpRequest {
    /// HTTP request method
    async fn method(&self) -> HttpMethod;
    /// HTTP request URL
    async fn url(&self) -> Url;
    /// HTTP request headers
    async fn headers(&self) -> HashMap<String, Vec<String>>;
    /// HTTP request body
    async fn body(&self) -> Vec<u8>;
}

/// Http Response without body
/// async?
pub trait HttpResponse {
    /// HTTP response version
    async fn version(&self) -> HttpVersion;
    /// HTTP response url
    async fn url(&self) -> Url;
    /// HTTP response status code
    async fn status(&self) -> u16;
    /// HTTP response status reason
    async fn reason(&self) -> String;
    /// HTTP response headers
    async fn headers(&self) -> HashMap<String, Vec<String>>;
    /// HTTP response body
    /// TODO: Result?
    async fn body(&self) -> Vec<u8>;

    /// Easy way to obtain HTTP response status category
    async fn status_category(&self) -> HttpResponseStatus {
        common_status_category(self.status().await)
    }
}

/// Http request as a struct ready to searilization
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HTTPRequest {
    /// HTTP request method
    pub method: HttpMethod,
    /// HTTP request URL
    pub url: Url,
    /// HTTP request headers
    pub headers: HashMap<String, Vec<String>>,
    /// HTTP request body
    pub body: Vec<u8>,
}

/// Http request as a struct ready to searilization
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HTTPResponse {
    /// HTTP response version
    pub version: HttpVersion,
    /// HTTP response url
    pub url: Url,
    /// HTTP response status code
    pub status: u16,
    /// HTTP response status reason
    pub reason: String,
    /// HTTP response headers
    pub headers: HashMap<String, Vec<String>>,
    /// HTTP response body
    // TODO: separate it out to an async trait/function/accessor
    pub body: Vec<u8>,
}

/// Collection of common HTTP response status categories
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum HttpResponseStatus {
    /// Informational Responses
    Status1xx,
    /// Successful Responses
    Status2xx,
    /// Redirection Responses
    Status3xx,
    /// Client Error responses
    Status4xx,
    /// Server Error responses
    Status5xx,
    /// Unknown HTTP responses
    StatusUnknown,
}

/// Represents an HTTP method
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(untagged, rename_all_fields = "UPPERCASE")]
pub enum HttpMethod {
    /// OPTIONS Http Method
    Options,
    /// GET Http Method
    Get,
    /// POST Http Method
    Post,
    /// PUT Http Method
    Put,
    /// DELETE Http Method
    Delete,
    /// HEAD Http Method
    Head,
    /// TRACE Http Method
    Trace,
    /// CONNECT Http Method
    Connect,
    /// PATCH Http Method
    Patch,
    /// Other custom Http Method (name provided)
    Custom(String),
}

/// Represents an HTTP version
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum HttpVersion {
    /// HTTP Version 0.9
    #[serde(rename = "HTTP/0.9")]
    Http09,
    /// HTTP Version 1.0
    #[serde(rename = "HTTP/1.0")]
    Http10,
    /// HTTP Version 1.1
    #[serde(rename = "HTTP/1.1")]
    Http11,
    /// HTTP Version 2.0
    #[serde(rename = "HTTP/2.0")]
    H2,
    /// HTTP Version 3.0
    #[serde(rename = "HTTP/3.0")]
    H3,
}

impl HTTPRequest {
    /// Easy constructor to create from arbitraty implementation
    pub async fn new(value: &impl HttpRequest) -> Self {
        HTTPRequest {
            method: value.method().await.clone(),
            url: value.url().await.clone(),
            headers: value.headers().await.clone(),
            body: value.body().await.clone(),
        }
    }
}

impl HttpRequest for HTTPRequest {
    #[doc = "HTTP request method"]
    async fn method(&self) -> HttpMethod {
        self.method.clone()
    }

    #[doc = "HTTP request URL"]
    async fn url(&self) -> Url {
        self.url.clone()
    }

    #[doc = "HTTP request headers"]
    async fn headers(&self) -> HashMap<String, Vec<String>> {
        self.headers.clone()
    }

    #[doc = "HTTP request body"]
    async fn body(&self) -> Vec<u8> {
        self.body.clone()
    }
}

impl HTTPResponse {
    /// Easy constructor to create from arbitraty implementation
    pub async fn new(value: &impl HttpResponse) -> Self {
        HTTPResponse {
            version: value.version().await.clone(),
            status: value.status().await,
            reason: value.reason().await.clone(),
            url: value.url().await.clone(),
            headers: value.headers().await.clone(),
            body: value.body().await.clone(),
        }
    }

    /// Easy way to obtain HTTP response status category
    async fn status_category(&self) -> HttpResponseStatus {
        common_status_category(self.status().await)
    }
}

impl HttpResponse for HTTPResponse {
    #[doc = "HTTP response version"]
    async fn version(&self) -> HttpVersion {
        self.version.clone()
    }

    #[doc = "HTTP response url"]
    async fn url(&self) -> Url {
        self.url.clone()
    }

    #[doc = "HTTP response status code"]
    async fn status(&self) -> u16 {
        self.status
    }

    #[doc = "HTTP response status reason"]
    async fn reason(&self) -> String {
        self.reason.clone()
    }

    #[doc = "HTTP response headers"]
    async fn headers(&self) -> HashMap<String, Vec<String>> {
        self.headers.clone()
    }

    #[doc = "HTTP response body"]
    async fn body(&self) -> Vec<u8> {
        self.body.clone()
    }
}

// save to JSON
// https://stackoverflow.com/questions/55653917/how-to-serialize-httpheadermap-into-json

fn common_status_category(status: u16) -> HttpResponseStatus {
    return match status {
        100..200 => HttpResponseStatus::Status1xx,
        200..300 => HttpResponseStatus::Status2xx,
        300..400 => HttpResponseStatus::Status3xx,
        400..500 => HttpResponseStatus::Status4xx,
        500..600 => HttpResponseStatus::Status5xx,
        _ => HttpResponseStatus::StatusUnknown,
    };
}
