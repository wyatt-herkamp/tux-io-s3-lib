use std::{fmt::Debug, sync::Arc};

use bytes::Bytes;
use http::StatusCode;
use reqwest::{Error, Request, RequestBuilder, Response};
use url::Url;

pub trait HttpClient: Send + Sync + Debug + Clone {
    fn get(&self, url: Url) -> RequestBuilder {
        self.request(reqwest::Method::GET, url)
    }

    fn post(&self, url: Url) -> RequestBuilder {
        self.request(reqwest::Method::POST, url)
    }

    fn put(&self, url: Url) -> RequestBuilder {
        self.request(reqwest::Method::PUT, url)
    }

    fn delete(&self, url: Url) -> RequestBuilder {
        self.request(reqwest::Method::DELETE, url)
    }

    fn request(&self, method: reqwest::Method, url: Url) -> RequestBuilder;

    fn execute(&self, request: Request) -> impl Future<Output = Result<Response, Error>> + Send;
}

impl HttpClient for reqwest::Client {
    fn request(&self, method: reqwest::Method, url: Url) -> RequestBuilder {
        self.request(method, url)
    }

    fn execute(&self, request: Request) -> impl Future<Output = Result<Response, Error>> + Send {
        self.execute(request)
    }
}
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub body: Bytes,
    pub status: StatusCode,
    pub headers: reqwest::header::HeaderMap,
}
#[derive(Debug, Clone)]
pub struct MockOkClient {
    response: MockResponse,
    request_counter: Arc<std::sync::atomic::AtomicUsize>,
}
impl MockOkClient {
    pub fn new(response: MockResponse) -> Self {
        Self {
            response,
            request_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
    pub fn request_count(&self) -> usize {
        self.request_counter
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}
impl HttpClient for MockOkClient {
    fn request(&self, method: reqwest::Method, url: Url) -> RequestBuilder {
        let client = reqwest::Client::new();
        client.request(method, url)
    }

    fn execute(&self, _request: Request) -> impl Future<Output = Result<Response, Error>> + Send {
        let response = self.response.clone();
        async move {
            self.request_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut http_response = http::Response::builder().status(response.status);
            for (key, value) in response.headers.iter() {
                http_response = http_response.header(key, value);
            }
            let http_response = http_response.body(response.body).unwrap();
            let response = Response::from(http_response);
            Ok(response)
        }
    }
}
