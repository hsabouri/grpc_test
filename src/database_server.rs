use std::collections::HashMap;
use std::sync::Arc;

use database_service::database_server::{Database, DatabaseServer};
use database_service::*;
use parking_lot::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use schema::*;

pub mod database_service {
    tonic::include_proto!("database"); // The string specified here must match the proto package name
}

/// Clearly not an ACID DB
#[derive(Default)]
pub struct MyDatabase {
    users: Arc<RwLock<(HashMap<UserId, User>, UserId)>>,
    tweets: Arc<RwLock<(HashMap<TweetId, Tweet>, TweetId)>>,
    tweets_by_user_index: Arc<RwLock<HashMap<UserId, Vec<TweetId>>>>,
}

impl MyDatabase {
    pub fn get_user(&self, id: UserId) -> Option<User> {
        self.users.read().0.get(&id).cloned()
    }

    pub fn add_user(&self) -> Option<User> {
        let mut db_state = self.users.write();

        let user_id = db_state.1 + 1;

        let new_user = User {
            user_id,
            following: vec![],
        };

        db_state.0.insert(new_user.user_id, new_user);
        db_state.1 = user_id;

        drop(db_state);

        self.get_user(user_id)
    }

    pub fn get_tweet(&self, id: TweetId) -> Option<Tweet> {
        self.tweets.read().0.get(&id).cloned()
    }

    pub fn add_tweet(&self, user_id: UserId, content: String) -> Option<Tweet> {
        let mut db_state = self.tweets.write();

        let tweet_id = db_state.1 + 1;

        let new_tweet = Tweet {
            tweet_id,
            user_id,
            content,
        };

        db_state.0.insert(new_tweet.tweet_id, new_tweet);
        db_state.1 = tweet_id;

        drop(db_state);

        // Update secondary index
        self._refresh_tweet_by_user_index(tweet_id, user_id);

        self.get_tweet(tweet_id)
    }

    fn _refresh_tweet_by_user_index(&self, tweet_id: TweetId, user_id: UserId) {
        let mut db_state = self.tweets_by_user_index.write();

        if let Some(index) = db_state.get_mut(&user_id) {
            index.push(tweet_id);
        } else {
            let mut index = vec![];

            index.push(tweet_id);
            db_state.insert(user_id, index);
        }
    }

    pub fn get_last_n_tweet_of_user(
        &self,
        user_id: UserId,
        count: usize,
    ) -> Result<Vec<Tweet>, String> {
        let db_state = self.tweets_by_user_index.read();

        let index = db_state
            .get(&user_id)
            .ok_or_else(|| format!("User does not exist"))?;

        let last_n_tweets: Vec<TweetId> = index.iter().rev().take(count).cloned().collect();

        drop(db_state);

        let last_n_tweets = last_n_tweets
            .into_iter()
            .map(|tweet_id| {
                self.get_tweet(tweet_id)
                    .ok_or_else(|| format!("Tweet {tweet_id} does not exist"))
            })
            .collect::<Result<Vec<Tweet>, String>>()?;

        Ok(last_n_tweets)
    }
}

#[tonic::async_trait]
impl Database for MyDatabase {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserReply>, Status> {
        println!("Got a request: {:?}", request);

        let user_id = request.into_inner().user_id;

        let user = self.get_user(user_id).ok_or_else(|| {
            Status::not_found(format!("User with ID '{}' does not exist.", user_id))
        })?;

        let reply = UserReply {
            user_id,
            following: user.following,
        };

        Ok(Response::new(reply))
    }

    async fn add_user(
        &self,
        request: Request<AddUserRequest>,
    ) -> Result<Response<UserReply>, Status> {
        println!("Got a request: {:?}", request);

        let user = self
            .add_user()
            .ok_or_else(|| Status::aborted("Failed to add user."))?;

        let reply = UserReply {
            user_id: user.user_id,
            following: user.following,
        };

        Ok(Response::new(reply))
    }

    async fn add_tweet(
        &self,
        request: Request<AddTweetRequest>,
    ) -> Result<Response<TweetReply>, Status> {
        println!("Got a request: {:?}", request);

        let tweet = request.into_inner();

        let tweet = self
            .add_tweet(tweet.user_id, tweet.content)
            .ok_or_else(|| Status::aborted("Failed to add tweet."))?;

        let reply = TweetReply {
            tweet_id: tweet.tweet_id,
            content: tweet.content,
        };

        Ok(Response::new(reply))
    }

    async fn get_last_tweets_of_user(
        &self,
        request: Request<GetLastTweetsOfUserRequest>,
    ) -> Result<Response<MultiTweetReply>, Status> {
        println!("Got a request: {:?}", request);

        let req = request.into_inner();

        let tweets = self
            .get_last_n_tweet_of_user(req.user_id, req.count as usize)
            .map_err(|e| Status::aborted(format!("Failed to get tweets: {e}")))?;

        let reply = MultiTweetReply {
            tweets: tweets
                .into_iter()
                .map(|tweet| TweetReply {
                    tweet_id: tweet.tweet_id,
                    content: tweet.content,
                })
                .collect(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyDatabase::default();

    Server::builder()
        .add_service(DatabaseServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
