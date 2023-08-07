mod tumblr;

use anyhow::*;
use env_logger::Env;
use futures::*;
use log::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_stream::StreamExt;
use tumblr::*;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let posts = tokio::fs::read("posts.json").await?;
    let mut posts: Vec<Post> = serde_json::from_slice(&posts)?;

    let stream = get_likes(None).await;
    pin_mut!(stream);
    let new_posts: Vec<Post> = stream
        .as_mut()
        .take_while(Result::is_ok)
        .try_collect()
        .await?;
    posts.extend(new_posts);

    if let Some(Err(err)) = stream.next().await {
        if let Some(ApiError::RateLimit { timestamp, .. }) = err.downcast_ref::<ApiError>() {
            if let Some(timestamp) = timestamp {
                info!("last returned timestamp: {}", timestamp);
            } else {
                info!("no final timestamp returned!");
            }
        }
    }

    tokio::fs::write(
        "posts.json",
        serde_json::to_string_pretty(posts.as_slice())?,
    )
    .await?;

    Ok(())
}

async fn get_likes(before: Option<String>) -> impl Stream<Item = Result<Post>> {
    stream::try_unfold(before, |token| async move {
        //debug!("{token:?}");
        let mut query = vec![
            (
                "api_key".to_string(),
                "yWH70O23rRJzAO69e6nj0lRUdrU7iCs8hCiUJZ6V7SM4TZGxIf".to_string(),
            ),
            //("limit".to_string(), "1".to_string()),
            ("npf".to_string(), "true".to_string()),
        ];
        if let Some(time) = &token {
            query.push(("before".to_string(), time.to_string()))
        };
        let client = Client::new();
        let resp = client
            .get("https://api.tumblr.com/v2/blog/manxome-foe/likes")
            .query(query.as_slice())
            .send()
            .inspect_err(|err| error!("{err:?}"))
            .await?;
        let status = resp.status();
        if !status.is_success() {
            error!("status returned: {status}");
            return Err(anyhow!(ApiError::RateLimit {
                status,
                timestamp: token,
            }));
        }
        let resp: TumblrResponse = resp.json().inspect_err(|err| error!("{err:?}")).await?;
        let resp = resp
            .response
            .context("no likes were returned with the response")?;
        let next_timestamp = resp
            .pagination
            .next
            .and_then(|next| next.query_params.before);
        debug!("next timestamp: {next_timestamp:?}");
        Ok(Some((
            stream::iter(resp.liked_posts.into_iter().map(Ok)),
            next_timestamp,
        )))
    })
    .try_flatten()
}
