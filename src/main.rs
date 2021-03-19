use egg_mode::Token;
use egg_mode::tweet::Tweet;
use futures::{StreamExt, Stream};

const DRY_RUN: bool = false;

async fn load_keypair(token_path: &str, secret_path: &str) -> tokio::io::Result<egg_mode::KeyPair> {
    let token = tokio::fs::read_to_string(token_path).await?;
    let secret = tokio::fs::read_to_string(secret_path).await?;

    Ok(egg_mode::KeyPair::new(token, secret))
}

fn user_timeline_stream(acct: String, with_replies: bool, with_rts: bool, token: &Token) -> impl Stream<Item=Vec<Tweet>> {
    let tl = egg_mode::tweet::user_timeline(acct, with_replies, with_rts, token);
    futures::stream::unfold(tl, |ttl| async move {
        let api_res = if ttl.max_id.is_none() {
            ttl.start()
        } else {
            ttl.older(None)
        };

        match api_res.await {
            Ok((ttl_next, tweets)) => {
                eprintln!("Got {} tweets", tweets.response.len());
                eprintln!("Pull {:?}", tweets.rate_limit_status);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Some((tweets.response, ttl_next))
            },
            Err(err) => {
                eprintln!("{:?}", err);
                None
            }
        }
    })
}

async fn delete_and_log(tweet: Tweet, token: &Token) {
    println!("{}", serde_json::to_string(&tweet).unwrap());
    if !DRY_RUN {
        let res = egg_mode::tweet::delete(tweet.id, &token).await;
        match res {
            Err(err) => eprintln!("Error deleting tweet: {:?}", err),
            Ok(resp) => eprintln!(">>> Delete {:?}", resp.rate_limit_status)
        }
    }
}

async fn start() {
    let cutoff_time = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(30)).unwrap();
    eprintln!("Deleting tweets older than {}", cutoff_time);

    let con_token = load_keypair("secrets/consumer_token", "secrets/consumer_secret").await.unwrap();
    let acc_token = load_keypair("secrets/access_token", "secrets/access_token_secret").await.unwrap();

    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: acc_token
    };

    let t = &token;

    user_timeline_stream("thing342".to_string(), true, true, t)
        .for_each_concurrent(None, |tweets| async move {
            let tweets_to_delete = tweets.into_iter()
                .filter(|t| t.created_at < cutoff_time);
            for tweet in tweets_to_delete {
                delete_and_log(tweet, t).await;
            }
        }).await;
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            start().await;
        })
}