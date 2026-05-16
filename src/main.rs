mod binder;
mod encoder;
mod handlers;
mod parser;
mod response;

use axum::{
    Router,
    routing::{get, post},
};
use handlers::{AppState, handle_create, handle_delete, handle_get, handle_list, handle_update};
use sqlx::mysql::MySqlPoolOptions;

use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set in .env");

    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to MySQL");

    let state = AppState { pool };

    let allow_origin_rule = match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins_str) if !origins_str.trim().is_empty() => {
            let origins: Vec<HeaderValue> = origins_str
                .split(',')
                .map(|s| s.trim().parse::<HeaderValue>())
                .filter_map(Result::ok)
                .collect();

            if origins.is_empty() {
                println!(
                    "⚠️ WARNING: CORS_ALLOWED_ORIGINS is configured but contains no valid URLs. CORS is disabled."
                );
                AllowOrigin::list(vec![])
            } else {
                println!("🔒 CORS Allowed Origins loaded: {:?}", origins_str);
                AllowOrigin::list(origins)
            }
        }
        _ => {
            println!(
                "⚠️ WARNING: CORS_ALLOWED_ORIGINS is not configured in .env. Defaulting to block all cross-origins."
            );
            AllowOrigin::list(vec![])
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(allow_origin_rule)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/v1/{table}", post(handle_create).get(handle_list))
        .route(
            "/api/v1/{table}/{id}",
            get(handle_get).put(handle_update).delete(handle_delete),
        )
        .with_state(state)
        .layer(cors);

    let bind_address = format!("0.0.0.0:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    println!(
        "🚀 Rust Zero-Code API Engine running on: http://{}",
        bind_address
    );

    axum::serve(listener, app).await.unwrap();
}
