mod grpc_client;

use std::env;

use crate::grpc_client::GrpcAuthClient;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/health-check", get(health_check))
        .route("/protected", get(protected));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

async fn root() -> impl IntoResponse {
    let address = env::var("AUTH_SERVICE_HOST").unwrap_or("".to_owned());
    if address.is_empty() {
        return Html("Internal error".to_owned());
    }

    let login_link = format!("{}", address);
    let logout_link = format!("{}/logout", address);

    let template = IndexTemplate {
        login_link,
        logout_link,
    };
    Html(template.render().unwrap())
}

async fn protected(jar: CookieJar) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let mut grpc_client = match GrpcAuthClient::new().await {
        Ok(client) => client,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match grpc_client
        .verify_token(jwt_cookie.value().to_string())
        .await
    {
        Ok(is_valid) => {
            if is_valid {
                return Json(ProtectedRouteResponse {
                    img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png"
                        .to_owned(),
                })
                .into_response();
            }
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    //
    // let api_client = reqwest::Client::builder().build().unwrap();
    //
    // let verify_token_body = serde_json::json!({
    //     "token": &jwt_cookie.value(),
    // });
    //
    // let auth_hostname = env::var("AUTH_SERVICE_HOST").unwrap_or("".to_owned());
    // if auth_hostname.is_empty() {
    //     return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    // }
    //
    // let url = format!("{}/verify-token", auth_hostname);
    //
    // let response = match api_client.post(&url).json(&verify_token_body).send().await {
    //     Ok(response) => response,
    //     Err(_) => {
    //         return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    //     }
    // };
    //
    // match response.status() {
    //     reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::BAD_REQUEST => {
    //         StatusCode::UNAUTHORIZED.into_response()
    //     }
    //     reqwest::StatusCode::OK => Json(ProtectedRouteResponse {
    //         img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
    //     })
    //     .into_response(),
    //     _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    // }
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}
