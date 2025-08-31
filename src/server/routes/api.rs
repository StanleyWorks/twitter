use std::{collections::HashMap, fs};

use axum::{
    Router,
    extract::{self, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use log::warn;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient, url::ParseError,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::api::client::{ApiClient, HttpClient};

#[derive(Debug, Clone)]
struct AppState {
    api_client: ApiClient,
}

impl std::ops::Deref for AppState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

#[derive(Deserialize)]
struct CreateTweet {
    text: String,
}

/// Handle API Routes
pub fn api_routes() -> Router {
    let api_client = ApiClient::new();
    let state = AppState { api_client };
    Router::new()
        .route("/health", get(health_check))
        .route("/tweet", post(create_tweet))
        .route("/generate-twitter", get(auth_url_handler))
        .route("/callback-twitter", get(handle_callback))
        .with_state(state)
}

/// Health check logic
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Ok".to_string())
}

/// Handle Twitter callback
async fn handle_callback(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let code = params.get("code").unwrap();
    let verifier_secret = fs::read_to_string("verifier_secret.txt").unwrap();
    let pkce_verifier = PkceCodeVerifier::new(verifier_secret);

    let access_token =
        exchange_code_for_token("cWFMbmZiT1hPbm1BTzg2bkJsYnc6MTpjaQ", code, pkce_verifier).await;

    match access_token {
        Ok(token) => {
            fs::write("access_token.txt", token).unwrap();
            (StatusCode::OK, "Authorized! You can now tweet.".to_string())
        }
        Err(err) => {
            warn!("Could not get access token. {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong in the server.".to_string(),
            )
        }
    }
}

/// Create a new tweet.
async fn create_tweet(
    State(mut state): State<AppState>,
    extract::Json(payload): extract::Json<CreateTweet>,
) -> impl IntoResponse {
    println!("Tweeting {}", payload.text);

    let access_token = fs::read_to_string("access_token.txt").unwrap();
    let res = state
        .api_client
        .with_bearer(&access_token)
        .post(
            "https://api.twitter.com/2/tweets",
            json!({
                "text": "This is a tweet."
            }),
        )
        .await;

    match res {
        Ok(success) => (StatusCode::OK, success),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something broke: {err}"),
        ),
    }
}

/// Generate auth url
async fn auth_url_handler() -> impl IntoResponse {
    let url = generate_auth_url(
        "cWFMbmZiT1hPbm1BTzg2bkJsYnc6MTpjaQ",
        "http://localhost:3000/api/callback-twitter",
    );

    let (full_url, pkce_verifier, _csrf) = url.unwrap();

    match fs::write("verifier_secret.txt", pkce_verifier.secret()) {
        Ok(_) => (StatusCode::OK, full_url),
        Err(err) => {
            warn!("Failed to write the secret: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Somethin broke!".to_string(),
            )
        }
    };
}

fn generate_auth_url(
    client_id: &str,
    redirect_uri: &str,
) -> Result<(String, PkceCodeVerifier, CsrfToken), ParseError> {
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_uri(AuthUrl::new(
            "https://twitter.com/i/oauth2/authorize".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("tweet.read".to_string()))
        .add_scope(Scope::new("tweet.write".to_string()))
        .add_scope(Scope::new("users.read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok((auth_url.to_string(), pkce_verifier, csrf_token))
}

async fn exchange_code_for_token(
    client_id: &str,
    auth_code: &str,
    pkce_verifier: PkceCodeVerifier,
) -> Result<String, String> {
    let client = BasicClient::new(ClientId::new(client_id.to_string())).set_token_uri(
        TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string())
            .map_err(|_| String::from("Failed to build client."))?,
    );

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| String::from("Failed to build client"))?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|err| {
            warn!("Could not get token: {err}");
            String::from("Could not get token")
        })?;

    Ok(token_result.access_token().secret().clone())
}
