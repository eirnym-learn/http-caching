use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future};
use url::Url;

use crate::core::Result;
/// Http Request status
pub trait HttpRequest: Send {
    /// HTTP request method
    fn method(&self) -> HttpMethod;
    /// HTTP request URL
    fn url(&self) -> Url;
    /// HTTP request headers
    fn headers(&self) -> HashMap<String, Vec<String>>;
    /// HTTP request body
    fn body(&self) -> Vec<u8>;
}

/// Http Response without body
pub trait HttpResponse: Send + Sync {
    /// HTTP response version
    fn version(&self) -> HttpVersion;
    /// HTTP response url
    fn url(&self) -> Url;
    /// HTTP response status code
    fn status(&self) -> u16;
    /// HTTP response status reason
    fn reason(&self) -> String;
    /// HTTP response headers
    fn headers(&self) -> HashMap<String, Vec<String>>;
    /// HTTP response body — called only when required
    fn body(&self) -> impl Future<Output = Result<Vec<u8>>> + Send + Sync;

    /// Easy way to obtain HTTP response status category
    fn status_category(&self) -> HttpResponseStatus {
        common_status_category(self.status())
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
    pub fn new(value: &impl HttpRequest) -> Self {
        HTTPRequest {
            method: value.method().clone(),
            url: value.url().clone(),
            headers: value.headers().clone(),
            body: value.body().clone(),
        }
    }
}

impl HttpRequest for HTTPRequest {
    #[doc = "HTTP request method"]
    fn method(&self) -> HttpMethod {
        self.method.clone()
    }

    #[doc = "HTTP request URL"]
    fn url(&self) -> Url {
        self.url.clone()
    }

    #[doc = "HTTP request headers"]
    fn headers(&self) -> HashMap<String, Vec<String>> {
        self.headers.clone()
    }

    #[doc = "HTTP request body"]
    fn body(&self) -> Vec<u8> {
        self.body.clone()
    }
}

impl HTTPResponse {
    /// Easy constructor to create from arbitraty implementation
    pub async fn new(value: &impl HttpResponse) -> Result<Self> {
        Ok(HTTPResponse {
            version: value.version().clone(),
            status: value.status(),
            reason: value.reason().clone(),
            url: value.url().clone(),
            headers: value.headers().clone(),
            body: value.body().await?.clone(),
        })
    }
    pub fn new_no_body(value: &impl HttpResponse) -> Self {
        HTTPResponse {
            version: value.version().clone(),
            status: value.status(),
            reason: value.reason().clone(),
            url: value.url().clone(),
            headers: value.headers().clone(),
            body: vec![],
        }
    }

    /// Easy way to obtain HTTP response status category
    pub fn status_category(&self) -> HttpResponseStatus {
        common_status_category(self.status())
    }
}

impl HttpResponse for HTTPResponse {
    #[doc = "HTTP response version"]
    fn version(&self) -> HttpVersion {
        self.version.clone()
    }

    #[doc = "HTTP response url"]
    fn url(&self) -> Url {
        self.url.clone()
    }

    #[doc = "HTTP response status code"]
    fn status(&self) -> u16 {
        self.status
    }

    #[doc = "HTTP response status reason"]
    fn reason(&self) -> String {
        self.reason.clone()
    }

    #[doc = "HTTP response headers"]
    fn headers(&self) -> HashMap<String, Vec<String>> {
        self.headers.clone()
    }

    #[doc = "HTTP response body"]
    fn body(&self) -> impl Future<Output = Result<Vec<u8>>> + Send {
        async { Ok(self.body.clone()) }
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
