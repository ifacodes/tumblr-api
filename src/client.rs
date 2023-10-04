use crate::auth::{self, Authentication, Token};
use anyhow::{anyhow, bail, Result};
use reqwest::Client;
use thiserror::Error;
#[derive(Debug)]
pub struct TumblrClient<A: Authentication> {
    client: Client,
    token: Option<A>,
}

impl<A: Authentication> TumblrClient<A> {
    fn new(token: A) -> Self {
        Self {
            client: Client::new(),
            token: Some(token),
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
struct Error(#[from] anyhow::Error);

/// Blog Methods
impl<A: Authentication> TumblrClient<A> {
    async fn info(&self, blog: &str) -> Result<()> {
        let res = self
            .client
            .get(format!("https://api.tumblr.com/v2/blog/{}/info", blog))
            .bearer_auth(
                self.token
                    .as_ref()
                    .ok_or_else(|| Error(anyhow!("no token")))?
                    .bearer_token(),
            )
            .send()
            .await?;
        println!("{:#?}", res.text().await?);
        Ok(())
    }
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
impl<A: Authentication> TumblrClient<A> {
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
impl<A: Authentication> TumblrClient<A> {
    fn tagged(&self) {}
}
