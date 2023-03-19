pub type UserId = i64;

#[derive(Clone)]
pub struct User {
    pub user_id: UserId,
    pub following: Vec<UserId>,
}

pub type TweetId = i64;

#[derive(Clone)]
pub struct Tweet {
    pub tweet_id: TweetId,
    pub user_id: UserId,
    pub content: String,
}
