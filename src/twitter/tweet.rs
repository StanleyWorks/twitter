use crate::{api::client::HttpClient, config::Config};
use oauth::{HMAC_SHA1, Token};
use reqwest::Error;
use serde_json::json;

use crate::{api::client::ApiClient, server::routes::api::CreateTweet};

pub async fn create(mut client: ApiClient, payload: CreateTweet) -> Result<String, Error> {
    let cfg = Config::load();

    let token = Token::from_parts(
        cfg.consumer_key,
        cfg.consumer_secret,
        cfg.access_token,
        cfg.access_secret,
    );

    let url = "https://api.twitter.com/2/tweets";
    let auth_header = oauth::post(url, &(), &token, HMAC_SHA1);
    println!("{}", auth_header);

    client
        .with_bearer(&auth_header)
        .post(url, json!({ "text": payload.text }))
        .await
}
