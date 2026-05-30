use axum::{
    body::Body,
    extract::Path,
    http::{header, Response, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rust_embed::Embed;
use tower_sessions::{MemoryStore, SessionManagerLayer};

mod auth;
mod cli;
mod config;
mod handlers;
mod templates;

#[derive(Embed)]
#[folder = "static/"]
struct Assets;

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());
    let cfg = config::load_config(&config_path)?;

    let state = AppState { config: cfg.clone() };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("agvpn_session")
        .with_http_only(true)
        .with_secure(false)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_path("/");

    let public = Router::new()
        .route("/login", get(handlers::login_page).post(handlers::login_submit));

    let protected = Router::new()
        .route("/", get(handlers::index_page))
        .route("/logout", post(handlers::logout))
        .route("/api/status", get(handlers::api_status))
        .route("/api/locations", get(handlers::api_locations))
        .route("/api/connect", post(handlers::api_connect))
        .route("/api/disconnect", post(handlers::api_disconnect))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .route("/static/{*path}", get(static_handler))
        .layer(session_layer)
        .with_state(state);

    let addr = format!("{}:{}", cfg.server.bind, cfg.server.port);
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
