use anyhow::{ensure, Result};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    *,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use url::Url;

async fn authenticate() -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
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
    std::env::set_var(
        "API_KEY",
        "yWH70O23rRJzAO69e6nj0lRUdrU7iCs8hCiUJZ6V7SM4TZGxIf",
    );
    std::env::set_var(
        "API_SECRET",
        "yvwu6G7C1TXv8dCbY5leERurQOY77sy9TK0g1NbdNFbZy1FWnP",
    );
    let token = authenticate().await;
    println!("{:#?}", token)
}
