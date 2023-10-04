use std::time::Duration;

use crate::tumblr::scope;
use ::reqwest::Client;
use anyhow::{anyhow, bail, ensure, Result};
use oauth2::{
    basic::{BasicClient, BasicTokenResponse, BasicTokenType},
    reqwest::async_http_client,
    *,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use url::Url;
pub trait Authentication {
    fn bearer_token(&self) -> &String;
    fn refresh_token(&self) -> Option<&String>;
}

pub struct Token {
    bearer_token: AccessToken,
    refresh_token: Option<RefreshToken>,
    expires_in: Option<Duration>,
    scopes: Vec<scope::Scope>,
}

#[derive(Error, Debug)]
#[error(transparent)]
struct Error(#[from] anyhow::Error);

impl TryFrom<BasicTokenResponse> for Token {
    type Error = anyhow::Error;
    fn try_from(value: BasicTokenResponse) -> Result<Self> {
        let bearer_token = value.access_token().clone();
        let refresh_token = value.refresh_token().cloned();
        let expires_in = value.expires_in();
        let scopes: Vec<scope::Scope> = value
            .scopes()
            .ok_or_else(|| Error(anyhow!("Missing Scopes")))?
            .iter()
            .map(|s| s.as_str().parse::<scope::Scope>())
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            bearer_token,
            refresh_token,
            expires_in,
            scopes,
        })
    }
}

impl Authentication for Token {
    fn bearer_token(&self) -> &String {
        self.bearer_token.secret()
    }

    fn refresh_token(&self) -> Option<&String> {
        self.refresh_token.as_ref().map(|rt| rt.secret())
    }
}

pub(crate) async fn authenticate(
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    let client = BasicClient::new(
        ClientId::new(std::env::var("API_KEY").expect("Missing API_KEY env variable.")),
        Some(ClientSecret::new(
            std::env::var("API_SECRET").expect("Missing API_SECRET env variable."),
        )),
        AuthUrl::new("https:/www.tumblr.com/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(
            TokenUrl::new("https:/api.tumblr.com/v2/oauth2/token".to_string())
                .expect("Invalid token endpoint URL"),
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080/".to_string()).expect("unable to set redirect url"),
    );

    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("basic".to_string()))
        .add_scope(Scope::new("offline_access".to_string()))
        .url();

    println!("Open this URL in your browser:\n{}\n", auth_url);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let code;
            {
                let mut reader = BufReader::new(&mut stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).await.unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let (key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let (key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                ensure!(csrf_state.secret() == CsrfToken::new(value.into_owned()).secret())
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).await?;

            // Exchange the code with a token.
            return client
                .exchange_code(code)
                .request_async(async_http_client)
                .await
                .map_err(anyhow::Error::from);
        }
    }
}

#[tokio::test]
async fn test_auth() {
    let token = authenticate().await;
    println!("{:#?}", token);

    let client = Client::new();
    let res = client
        .get("https://api.tumblr.com/v2/blog/manxomefoe.tumblr.com/info")
        .bearer_auth(token.unwrap().access_token().secret())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{:#?}", res)
}
