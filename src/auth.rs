use crate::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let is_authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if is_authenticated {
        return next.run(request).await;
    }

    let is_htmx = request
        .headers()
        .get("HX-Request")
        .is_some_and(|v| v == "true");

    if is_htmx {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            [("HX-Redirect", "login")],
            "",
        )
            .into_response()
    } else {
        Redirect::to("login").into_response()
    }
}
