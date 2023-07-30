use super::Error;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

#[derive(Clone)]
pub struct HttpClient {
    inner_client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl HttpClient {
    pub fn new(inner_client: hyper::Client<HttpsConnector<HttpConnector>>) -> Self {
        Self { inner_client }
    }

    pub fn inner(&self) -> &hyper::Client<HttpsConnector<HttpConnector>> {
        &self.inner_client
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let response = self
            .inner_client
            .get(url.parse().expect("Invalid URL"))
            .await?;
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            return Err(Error::HttpError(status));
        }
        let body = response.into_body();
        let bytes = hyper::body::to_bytes(body).await?;
        println!("Response Body: {}", String::from_utf8_lossy(&bytes));
        Ok(serde_json::from_slice(&bytes)?)
    }
}
