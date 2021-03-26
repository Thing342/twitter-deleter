use egg_mode::Token;
use egg_mode::tweet::Tweet;
use futures::{StreamExt, Stream};
use std::error::Error;
use chrono::{DateTime, Utc};
use std::ffi::OsStr;
use std::str::FromStr;

async fn load_keypair(token_path: &str, secret_path: &str) -> tokio::io::Result<egg_mode::KeyPair> {
    let token = tokio::fs::read_to_string(token_path).await?;
    let secret = tokio::fs::read_to_string(secret_path).await?;

    Ok(egg_mode::KeyPair::new(token, secret))
}

fn env_var_or_default<K: AsRef<OsStr>, F: FromStr>(key: K, default: F) -> Result<F, F::Err> {
    match std::env::var(key) {
        Ok(ev) => ev.parse(),
        Err(_) => Ok(default)
    }
}

fn env_var_or_default_str<K: AsRef<OsStr>>(key: K, default: &str) -> String {
    match std::env::var(key) {
        Ok(ev) => ev,
        Err(_) => default.to_string()
    }
}

#[derive(Debug)]
struct Config {
    dry_run: bool,
    username: String,
    token: egg_mode::Token,
    delete_before: DateTime<Utc>
}

impl Config {
    async fn load() -> Result<Config, Box<dyn Error>> {
        let secrets_volume = env_var_or_default_str("SECRETS_DIR", "./secrets");
        let dry_run = env_var_or_default("DRY_RUN", true).expect("failed to parse DRY_RUN");
        let days = env_var_or_default("DAYS_TO_KEEP", 30).expect("failed to parse DAYS_TO_KEEP");

        let con_token = load_keypair(&format!("{}/consumer_token", secrets_volume), &format!("{}/consumer_secret", secrets_volume)).await?;
        let acc_token = load_keypair(&format!("{}/access_token", secrets_volume), &format!("{}/access_token_secret", secrets_volume)).await?;
        let username = tokio::fs::read_to_string(format!("{}/username", secrets_volume)).await?;

        let token = egg_mode::Token::Access {
            consumer: con_token,
            access: acc_token
        };

        let delete_before = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(days)).unwrap();
        
        Ok(Config { dry_run, username, token, delete_before })
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn should_delete(&self, t: &Tweet) -> bool {
        t.created_at < self.delete_before
    }

    async fn delete_and_log(&self, tweet: Tweet) {
        println!("{}", serde_json::to_string(&tweet).unwrap());
        if !self.dry_run {
            let res = egg_mode::tweet::delete(tweet.id, &self.token).await;
            match res {
                Err(err) => eprintln!("Error deleting tweet: {:?}", err),
                Ok(resp) => eprintln!(">>> Delete {:?}", resp.rate_limit_status)
            }
        }
    }
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
                if tweets.response.len() == 0 {
                    eprintln!("No tweets left! Ending stream.");
                    None
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    Some((tweets.response, ttl_next))
                }
            },
            Err(err) => {
                eprintln!("{:?}", err);
                None
            }
        }
    })
}

async fn start() {
    let config = Config::load().await.expect("Failed to load Config.");

    eprintln!("Dry run! Printing tweets that would be deleted.");
    eprintln!("Deleting tweets older than {}", config.delete_before);

    let configg = &config;

    user_timeline_stream(config.username.to_string(), true, true, config.token())
        .for_each_concurrent(None, |tweets| async move {
            let tweets_to_delete = tweets.into_iter()
                .filter(|t| configg.should_delete(t));
            for tweet in tweets_to_delete {
                configg.delete_and_log(tweet).await;
            }
        }).await;

    eprintln!("Deletion completed.");
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