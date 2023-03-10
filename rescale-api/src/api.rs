use std::{sync::Arc, fmt::Display};

use reqwest::{Method, IntoUrl};

#[derive(Debug)]
pub struct RescaleApiClient {
    pub(crate) client: reqwest::Client,
    pub(crate) token: RescaleApiToken,
    pub(crate) api_url: String,
}

impl RescaleApiClient {
    pub fn new(token: &str, api_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: RescaleApiToken::new(token),
            api_url,
        }
    }

    pub fn new_eu(token: &str) -> Self {
        Self::new(token, "https://eu.rescale.com/api/v2/".to_string())
    }

    pub fn new_platform(token: &str) -> Self {
        Self::new(token, "https://platform.rescale.com/api/v2/".to_string())
    }

    pub fn request_inner<U: IntoUrl>(&self, method: Method, url: U) -> reqwest::RequestBuilder {
        self.client.request(method, url)
            .header("Authorization", self.token.header_value())
    }

    // TODO: This regularly double allocs, once in this method body and once in the caller.
    pub fn request(&self, method: Method, url: impl Display) -> reqwest::RequestBuilder {
        self.request_inner(method, format!("{}{}", self.api_url, url))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RescaleApiToken {
    token: String,
}

impl RescaleApiToken {
    pub fn new(token: &str) -> Self {
        Self {
            token: format!("Token {token}"),
        }
    }

    pub fn header_value(&self) -> &str {
        &self.token
    }
}
