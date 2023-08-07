use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Blog {
    name: String,
    title: Option<String>,
    url: Url,
    description: Option<String>,
    uuid: String,
    updated: Option<usize>,
    can_show_badges: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Post {
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
        content: Vec<Content>,
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
        formatting: Option<Vec<InlineFormat>>,
    },
    Image {
        media: Vec<MediaObject>,
        //colors: Option<HashMap<String, Value>>,
        feedback_token: Option<String>,
        poster: Option<Poster>,
        alt_text: Option<String>,
        caption: Option<String>,
    },
    Link {
        url: Option<Url>,
        title: Option<String>,
        description: Option<String>,
        author: Option<String>,
        site_name: Option<String>,
        display_url: Option<String>,
        //poster: Option<Poster>,
    },
    Audio {
        url: Option<Url>,
        media: Option<MediaObject>,
        provider: Option<String>,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        //poster: Option<Poster>,
        embed_html: Option<String>,
        embed_url: Option<Url>,
    },
    Video {
        url: Option<Url>,
        media: Option<MediaObject>,
        provider: Option<String>,
        embed_html: Option<String>,
        //embed_iframe: Option<EmbedIFrameObject>,
        embed_url: Option<Url>,
        // poster: Option<Poster>,
        // can_autoplay_on_cellular: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum InlineFormat {
    Bold {
        start: usize,
        end: usize,
    },
    Italic {
        start: usize,
        end: usize,
    },
    StrikeThrough {
        start: usize,
        end: usize,
    },
    Small {
        start: usize,
        end: usize,
    },
    Link {
        start: usize,
        end: usize,
        url: Url,
    },
    Mention {
        start: usize,
        end: usize,
        blog: Blog,
    },
    Color {
        start: usize,
        end: usize,
        hex: String,
    },
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
    media_key: Option<String>,
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
pub struct TumblrResponse {
    meta: Meta,
    pub response: Option<Liked>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Liked {
    liked_posts: Vec<Post>,
    liked_count: usize,
    #[serde(rename = "_links")]
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    pub next: Option<PaginationResult>,
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

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("received rate limit response")]
    RateLimit {
        status: StatusCode,
        timestamp: Option<String>,
    },
}
