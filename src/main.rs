use anyhow::*;
use futures::*;
use log::*;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, collections::HashMap, error::Error};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Blog {
    name: String,
    title: String,
    url: Url,
    description: String,
    uuid: String,
    updated: usize,
    can_show_badges: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum Post {
    Blocks {
        is_blocks_post_format: bool,
        blog_name: String,
        blog: Blog,
        id: usize,
        id_string: String,
        is_blazed: bool,
        is_blaze_pending: bool,
        post_url: Url,
        slug: String,
        date: String,
        timestamp: usize,
        state: String,
        reblog_key: String,
        tags: Vec<String>,
        short_url: Url,
        summary: String,
        should_open_in_legacy: bool,
        // recommended_source: Option<>,
        // recommened_color: Option<>,
        note_count: usize,
        // content: Vec<Content>,
        // layout: Vec<Layout>,
        // trail: Vec<Trail>,
        liked_timestamp: usize,
        can_like: bool,
        interactability_reblog: String,
        interactability_blaze: String,
        can_reblog: bool,
        can_send_in_message: bool,
        can_reply: bool,
        display_avatar: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum Content {
    Text {
        text: String,
        subtype: Option<String>,
        indent_level: Option<usize>,
        formatting: Vec<InlineFormat>,
    },
    Image {
        media: Vec<MediaObject>,
        colors: Option<HashMap<String, Value>>,
        feedback_token: Option<String>,
        poster: Option<Poster>,
        alt_text: Option<String>,
        caption: Option<String>,
    },
    Link {
        url: Url,
        title: String,
        description: String,
        author: String,
        site_name: Option<String>,
        display_url: Option<String>,
        poster: Option<Poster>,
    },
    Audio {
        url: Option<Url>,
        media: Box<Option<MediaObject>>,
        provider: Option<String>,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        poster: Option<Poster>,
        embed_html: Option<String>,
        embed_url: Option<Url>,
    },
    Video {
        url: Option<Url>,
        media: Box<Option<MediaObject>>,
        provider: Option<String>,
        embed_html: Option<String>,
        embed_iframe: Option<EmbedIFrameObject>,
        embed_url: Option<Url>,
        poster: Option<Poster>,
        can_autoplay_on_cellular: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum InlineFormat {
    Bold(Range),
    Italic(Range),
    StrikeThrough(Range),
    Small(Range),
    Link {
        #[serde(flatten)]
        range: Range,
        url: Url,
    },
    Mention {
        #[serde(flatten)]
        range: Range,
        blog: Blog,
    },
    Color {
        #[serde(flatten)]
        range: Range,
        hex: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Range {
    start: usize,
    end: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Poster {
    #[serde(rename = "type")]
    ty: String,
    url: Url,
    widht: usize,
    height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Layout {
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trail {
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MediaObject {
    media_key: String,
    url: Url,
    #[serde(rename = "type")]
    ty: Option<String>,
    width: Option<usize>,
    height: Option<usize>,
    original_dimensions_missing: Option<bool>,
    cropped: Option<bool>,
    has_original_dimensions: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbedIFrameObject {
    url: Url,
    width: usize,
    height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Meta {
    status: usize,
    msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TumblrResponse {
    meta: Meta,
    response: Option<Liked>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Liked {
    liked_posts: Vec<Post>,
    liked_count: usize,
    #[serde(rename = "_links")]
    pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    next: Option<PaginationResult>,
    prev: Option<PaginationResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PaginationResult {
    href: String,
    method: String,
    query_params: QueryParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryParams {
    npf: Option<String>,
    before: Option<String>,
    after: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let stream = get_likes(None).await;

    let posts: Vec<Post> = stream.collect().await; //println!("{}", serde_json::to_string_pretty(&res_json.response)?);

    tokio::fs::write("posts.json", serde_json::to_string_pretty(&posts)?).await?;

    Ok(())
}

async fn get_likes(before: Option<String>) -> impl Stream<Item = Post> {
    stream::unfold(before, |token| async move {
        let mut query = vec![
            (
                "api_key".to_string(),
                "yWH70O23rRJzAO69e6nj0lRUdrU7iCs8hCiUJZ6V7SM4TZGxIf".to_string(),
            ),
            ("npf".to_string(), "true".to_string()),
        ];
        if let Some(time) = token {
            query.push(("before".to_string(), time))
        };
        let client = Client::new();
        client
            .get("https://api.tumblr.com/v2/blog/manxome-foe/likes")
            .query(query.as_slice())
            .send()
            .map_err(anyhow::Error::from)
            .and_then(|res| async move {
                let status = res.status();
                res.bytes()
                    .map_err(anyhow::Error::from)
                    .and_then(|res_bytes| async move {
                        //let _ = tokio::fs::write("response2.json", &res_bytes).await;
                        if status.is_success() {
                            debug!("Status Returned: {}", status);
                            //info!("{:?}", json.response);
                            serde_json::from_slice::<TumblrResponse>(&res_bytes)
                                .map_err(|e| {
                                    error!("{:?}", &e);
                                    anyhow::Error::from(e)
                                })
                                .and_then(|tr| tr.response.context("no response"))
                        } else {
                            error!("Status Returned: {}", status);
                            bail!("ohno")
                        }
                    })
                    .map_ok(|liked| {
                        let before = if let Some(next) = liked.pagination.next {
                            next.query_params.before
                        } else {
                            None
                        };
                        info!("{:?}", before);
                        (stream::iter(liked.liked_posts), before)
                    })
                    .await
            })
            .await
            .ok()
    })
    .flatten()
}
