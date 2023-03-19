use database_service::{database_client::DatabaseClient, GetUserRequest};
use futures::{stream::FuturesUnordered, StreamExt};
use rand::seq::IteratorRandom;
use tonic::transport::Channel;

use crate::database_service::*;

pub mod database_service {
    tonic::include_proto!("database"); // The string specified here must match the proto package name
}

async fn add_tweet_request(
    client: &DatabaseClient<Channel>,
    req: AddTweetRequest,
) -> Result<tonic::Response<TweetReply>, tonic::Status> {
    let mut channel = client.clone();

    channel.add_tweet(req).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DatabaseClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(AddUserRequest {});

    let response = client.add_user(request).await?;
    let response = client
        .get_user(GetUserRequest {
            user_id: response.into_inner().user_id,
        })
        .await?;

    let user = response.into_inner();

    println!("User: {:#?}", user);

    let tweets = vec![AddTweetRequest {
        user_id: user.user_id,
        content: "Just had the best coffee ever ☕️ #coffeeaddict".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Excited to announce my new website is live! 🎉 #webdesign".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Feeling grateful for my amazResult<tonic::Response<super::TweetReply>, tonic::Status>
        that came out last weekend 🎥 #movienight".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Just started learning a new programming language and it's so exciting! 🤓 #codinglife".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Feeling inspired after reading a great book 📚 #bookworm".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Had an amazing time at the beach today 🏖️ #beachlife".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Feeling a bit under the weather today 😷 Any remedies?".into(),
    },
    AddTweetRequest {
        user_id: user.user_id,
        content: "Just tried a new restaurant and it was delicious! 🍴 #foodie".into(),
    }];

    let mut rng = rand::thread_rng();

    let futs = tweets
        .into_iter()
        .choose_multiple(&mut rng, 5)
        .into_iter()
        .map(|tweet| add_tweet_request(&client, tweet));

    FuturesUnordered::from_iter(futs)
        .for_each(|response| async {
            match response {
                Ok(res) => {
                    let tweet = res.into_inner();

                    println!("Tweet successfully added in DB : {}", tweet.tweet_id);
                }
                Err(err) => println!("Tweet failed to be added in DB : {:#?}", err),
            }
        })
        .await;

    Ok(())
}
