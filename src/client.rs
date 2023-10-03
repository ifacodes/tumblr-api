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

/// Blog Methods
impl TumblrClient {
    fn info(&self) {}
    fn avatar(&self) {}
    fn blocks(&self) {}
    fn block(&self) {}
    fn block_bulk(&self) {}
    fn likes(&self) {}
    fn following(&self) {}
    fn followers(&self) {}
    fn followed_by(&self) {}
    fn posts(&self) {}
    fn posts_queue(&self) {}
    fn posts_queue_reorder(&self) {}
    fn posts_queue_shuffle(&self) {}
    fn posts_drafts(&self) {}
    fn posts_submission(&self) {}
    fn notifications(&self) {}
    fn post(&self) {}
    fn post_edit(&self) {}
    fn post_reblog(&self) {}
    fn posts_create_neue(&self) {}
    fn fetch_post(&self) {}
    fn edit_post(&self) {}
    fn delete_post(&self) {}
    fn mute_post_notification(&self) {}
    fn post_notes(&self) {}
}

/// User Methods
impl TumblrClient {
    fn user_info(&self) {}
    fn user_limits(&self) {}
    fn user_dashboard(&self) {}
    fn user_likes(&self) {}
    fn user_following(&self) {}
    fn user_follow(&self) {}
    fn user_unfollow(&self) {}
    fn user_like(&self) {}
    fn user_unlike(&self) {}
    fn user_filtered_tags(&self) {}
    fn user_filtered_content(&self) {}
}

/// Tagged Method
impl TumblrClient {
    fn tagged(&self) {}
}
