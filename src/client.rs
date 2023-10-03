use reqwest::{Client, ClientBuilder};

#[derive(Debug, Default)]
pub struct TumblrClient {
    client: Client,
    consumer_key: &'static str,
    consumer_secret: &'static str,
    token: Option<()>,
    token_secret: Option<()>,
}

impl TumblrClient {
    fn new() -> Self {
        Self::default()
    }
}
