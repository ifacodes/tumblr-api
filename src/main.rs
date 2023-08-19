mod auth;
mod tumblr;
use anyhow::*;
use clap::*;
use env_logger::Env;
use futures::{StreamExt, *};
use log::*;
use reqwest::Client;
use std::env;
use tumblr::*;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    timestamp: Option<String>,
    #[arg(short, long)]
    download: bool,
    #[arg(short, long)]
    user: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let args = Args::parse();
    let posts = tokio::fs::read("posts.json").await?;
    let mut posts: Vec<Post> = serde_json::from_slice(&posts)?;

    if args.download {
        let media: Vec<(String, String, MediaObject)> = posts
            .into_iter()
            .flat_map(|post| match post {
                Post::Blocks {
                    content,
                    blog_name,
                    date,
                    ..
                } => content
                    .into_iter()
                    .flat_map(|content| match content {
                        Content::Image { media, .. } => {
                            Some((blog_name.clone(), date.clone(), media[0].clone()))
                        }
                        _ => None,
                    })
                    .collect::<Vec<(String, String, MediaObject)>>(),
            })
            .collect();

        let stream = stream::iter(media);
        stream
            .for_each_concurrent(50, |(blog_name, date, media)| async move {
                let year = date[0..4].to_string();
                let url = media.url;
                let res = reqwest::get(url.as_str()).await;
                tokio::spawn(async move {
                    let res = res?.bytes().await?;
                    info!(
                        "{}",
                        format!("saving to: images/{}/{}/{}", year, blog_name, url)
                    );
                    let _ =
                        tokio::fs::create_dir_all(format!("images/{}/{}", year, blog_name)).await;
                    tokio::fs::write(
                        format!(
                            "images/{}/{}/{}",
                            year,
                            blog_name,
                            url.path_segments().unwrap().last().unwrap()
                        ),
                        &res,
                    )
                    .await?;
                    Ok(())
                });
            })
            .await;
    } else {
        let stream = get_likes(args.timestamp, &args.user).await;
        pin_mut!(stream);
        let new_posts: Vec<Post> =
            tokio_stream::StreamExt::take_while(stream.as_mut(), Result::is_ok)
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
    }

    Ok(())
}

async fn get_likes(before: Option<String>, user: &str) -> impl Stream<Item = Result<Post>> + '_ {
    stream::try_unfold(before, move |token| async move {
        let mut query = vec![
            ("api_key".to_string(), std::env::var("API_KEY").unwrap()),
            ("npf".to_string(), "true".to_string()),
        ];
        if let Some(time) = &token {
            query.push(("before".to_string(), time.to_string()))
        };
        let client = Client::new();
        let resp = client
            .get(format!("https://api.tumblr.com/v2/blog/{}/likes", user))
            .query(query.as_slice())
            .send()
            .inspect_err(|err| error!("response error: {err:?}"))
            .await?;
        let status = resp.status();
        if !status.is_success() {
            error!("status returned: {status}");
            return Err(anyhow!(ApiError::RateLimit {
                status,
                timestamp: token,
            }));
        }
        let resp: TumblrResponse = resp
            .json()
            .inspect_err(|err| error!("unable to parse to TumblrResponse struct: {err:?}"))
            .await?;
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
