use crate::cli;
use crate::templates::{IndexTemplate, LoginTemplate};
use crate::AppState;
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use tower_sessions::Session;

#[derive(Deserialize)]
pub struct LoginForm {
    pub password: String,
}

#[derive(Deserialize)]
pub struct ConnectForm {
    pub location: String,
}

pub async fn login_page() -> impl IntoResponse {
    let tmpl = LoginTemplate { error: None };
    Html(tmpl.render().unwrap())
}

pub async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    if form.password == state.config.auth.password {
        session.insert("authenticated", true).await.ok();
        Redirect::to("./").into_response()
    } else {
        let tmpl = LoginTemplate {
            error: Some("密码错误".to_string()),
        };
        Html(tmpl.render().unwrap()).into_response()
    }
}

pub async fn logout(session: Session) -> Response {
    session.delete().await.ok();
    Redirect::to("./login").into_response()
}

pub async fn index_page() -> impl IntoResponse {
    let tmpl = IndexTemplate;
    Html(tmpl.render().unwrap())
}

pub async fn api_status(State(state): State<AppState>) -> impl IntoResponse {
    match cli::get_status(&state.config.vpn.cli_path).await {
        Ok(status) => {
            let status_class = if status.connected { "connected" } else { "disconnected" };
            let status_text = if status.connected { "已连接" } else { "未连接" };
            let location_html = match &status.location {
                Some(loc) => format!("<span class=\"location\">{}</span>", html_escape(loc)),
                None => String::new(),
            };
            Html(format!(
                r#"<div class="status-badge {status_class}">{status_text}</div>{location_html}"#
            ))
        }
        Err(e) => Html(format!(
            "<span class=\"error\">状态获取失败: {}</span>",
            html_escape(&e.to_string())
        )),
    }
}

pub async fn api_locations(State(state): State<AppState>) -> Response {
    let locations = match cli::list_locations(&state.config.vpn.cli_path).await {
        Ok(locs) => locs,
        Err(e) => {
            return Html(format!(
                "<tr><td colspan=\"5\" class=\"error\">获取列表失败: {}</td></tr>",
                html_escape(&e.to_string())
            ))
            .into_response();
        }
    };

    let current_location = cli::get_status(&state.config.vpn.cli_path)
        .await
        .ok()
        .and_then(|s| s.location);

    let mut html = String::new();
    for loc in &locations {
        let is_current = current_location.as_ref().is_some_and(|cl| {
            let cl_lower = cl.to_lowercase();
            let city_lower = loc.city.to_lowercase();
            let name_lower = loc.name.to_lowercase();
            cl_lower == city_lower
                || cl_lower.contains(&city_lower)
                || city_lower.contains(&cl_lower)
                || cl_lower == name_lower
        });

        let row_class = if is_current { "current" } else { "" };
        let action_btn = if is_current {
            r##"<button class="btn btn-disconnect" hx-post="api/disconnect" hx-target="#status" hx-swap="innerHTML" hx-trigger="click">断开</button>"##.to_string()
        } else {
            format!(
                r##"<button class="btn btn-connect" hx-post="api/connect" hx-vals='{{"location":"{}"}}' hx-target="#status" hx-swap="innerHTML" hx-trigger="click">连接</button>"##,
                html_escape(&loc.city)
            )
        };

        html.push_str(&format!(
            r#"<tr class="{row_class}"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{action_btn}</td></tr>"#,
            html_escape(&loc.name),
            html_escape(&loc.country),
            html_escape(&loc.city),
            loc.ping.as_deref().unwrap_or("-"),
        ));
    }

    let mut headers = HeaderMap::new();
    if current_location.is_some() {
        headers.insert(
            "HX-Trigger",
            HeaderValue::from_static("locations-changed"),
        );
    }

    (headers, Html(html)).into_response()
}

pub async fn api_connect(
    State(state): State<AppState>,
    Form(form): Form<ConnectForm>,
) -> Response {
    match cli::connect(&state.config.vpn.cli_path, &form.location).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Trigger",
                HeaderValue::from_static("locations-changed"),
            );
            (
                headers,
                Html(format!(
                    "<div class=\"status-badge connected\">已连接</div><span class=\"location\">{}</span>",
                    html_escape(&form.location)
                )),
            )
                .into_response()
        }
        Err(e) => Html(format!(
            "<span class=\"error\">连接失败: {}</span>",
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

pub async fn api_disconnect(State(state): State<AppState>) -> Response {
    match cli::disconnect(&state.config.vpn.cli_path).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "HX-Trigger",
                HeaderValue::from_static("locations-changed"),
            );
            (
                headers,
                Html("<div class=\"status-badge disconnected\">未连接</div>"),
            )
                .into_response()
        }
        Err(e) => Html(format!(
            "<span class=\"error\">断开失败: {}</span>",
            html_escape(&e.to_string())
        ))
        .into_response(),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
