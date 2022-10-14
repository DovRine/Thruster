use std::collections::HashMap;

use crate::app::App;

use crate::context::hyper_request::HyperRequest;
use crate::core::context::Context;
use async_trait::async_trait;
use hyper::body::HttpBody;
use hyper::{body, Body, Request, Response};

use super::Testable;

pub async fn request<
    B: HttpBody + Send + std::marker::Unpin,
    T: Context<Response = Response<B>> + Clone + Send + Sync,
    S: 'static + Send,
>(
    app: &App<HyperRequest, T, S>,
    request: Request<Body>,
) -> TestResponse {
    let uri = request.uri().to_string();
    let matched_route = app.resolve_from_method_and_path(request.method().as_str(), uri);
    let response = app
        .resolve(HyperRequest::new(request), matched_route)
        .await
        .unwrap();

    TestResponse::new(response).await
}

#[async_trait]
impl<T: Context<Response = Response<Body>> + Clone + Send + Sync, S: 'static + Send + Sync> Testable
    for App<HyperRequest, T, S>
{
    async fn get(
        &self,
        route: String,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        let response = request(self, req.body(Body::default())?).await;

        Ok(response)
    }
    async fn option(
        &self,
        route: String,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        Ok(request(self, req.body(Body::default())?).await)
    }
    async fn post(
        &self,
        route: String,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        Ok(request(self, req.body(body)?).await)
    }

    async fn put(
        &self,
        route: String,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        Ok(request(self, req.body(body)?).await)
    }

    async fn delete(
        &self,
        route: String,
        headers: Vec<(String, String)>,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        Ok(request(self, req.body(Body::default())?).await)
    }

    async fn patch(
        &self,
        route: String,
        headers: Vec<(String, String)>,
        body: Body,
    ) -> Result<TestResponse, Box<dyn std::error::Error>> {
        let mut req = Request::builder().uri(route);

        for header in headers {
            req = req.header(&header.0, &header.1);
        }

        Ok(request(self, req.body(body)?).await)
    }
}

#[derive(Debug)]
pub struct TestResponse {
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub trailers: Option<HashMap<String, String>>,
    pub status: u16,
}

impl TestResponse {
    async fn new<B: HttpBody + Send + std::marker::Unpin>(
        mut response: Response<B>,
    ) -> TestResponse {
        let mut headers = vec![];

        for (key, value) in response.headers().iter() {
            headers.push((key.as_str().to_owned(), value.to_str().unwrap().to_owned()));
        }

        let status = response.status().as_u16();
        let mut trailers = None;

        match response.trailers().await {
            Ok(res) => {
                // Refactor this to use optional methods
                if let Some(res) = res {
                    let mut trailer_map = HashMap::new();

                    for (key, value) in res.iter() {
                        trailer_map
                            .insert(key.as_str().to_owned(), value.to_str().unwrap().to_owned());
                    }
                    trailers = Some(trailer_map);
                }
            }
            Err(_) => panic!("Could not correctly get trailers"),
        }

        let body = response.into_body();
        let body = match body::to_bytes(body).await {
            Ok(res) => res,
            Err(_) => panic!("Could not correctly parse response"),
        }
        .to_vec();

        TestResponse {
            body,
            headers,
            trailers,
            status,
        }
    }

    pub fn body_string(&self) -> String {
        std::str::from_utf8(&self.body).unwrap().to_string()
    }
}
