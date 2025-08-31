use reqwest::Client;

pub trait HttpClient {
    fn new() -> Self;
    fn get(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<String, reqwest::Error>> + Send;
    fn post(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<String, reqwest::Error>> + Send;
}

struct Response {
    status: u16,
    content: String,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    bearer_token: String,
}

impl ApiClient {
    pub fn with_bearer(&mut self, token: &str) -> &Self {
        self.bearer_token = token.to_string();

        self
    }
}

impl HttpClient for ApiClient {
    fn new() -> Self {
        let client = Client::new();
        let bearer_token = String::new();
        Self {
            client,
            bearer_token,
        }
    }

    async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        let res = self.client.get(url).send().await?.text().await?;

        Ok(res)
    }

    async fn post(&self, url: &str, body: serde_json::Value) -> Result<String, reqwest::Error> {
        let res = self
            .client
            .post(url)
            .bearer_auth(&self.bearer_token)
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_api_client() {
        struct MockClient {}

        impl HttpClient for MockClient {
            fn new() -> Self {
                Self {}
            }

            async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
                Ok(format!("GET {url}").to_string())
            }

            async fn post(
                &self,
                url: &str,
                _body: serde_json::Value,
            ) -> Result<String, reqwest::Error> {
                Ok(format!("POST {url}").to_string())
            }
        }

        let http_client = MockClient::new();

        let result = http_client.get("https://stanleymasinde.com").await.unwrap();
        assert_eq!(result, "GET https://stanleymasinde.com".to_string())
    }
}
