use axum::{
    extract::Json,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Deserialize)]
struct CalcRequest {
    hourly_rate: f64,
    hours_per_week: f64,
    weeks_per_month: Option<f64>,
}

#[derive(Serialize)]
struct CalcResponse {
    monthly_hours: f64,
    monthly_gross: f64,
    weekly_gross: f64,
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn calc(Json(req): Json<CalcRequest>) -> impl IntoResponse {
    if req.hourly_rate < 0.0 || req.hours_per_week < 0.0 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"valeurs negatives interdites"}))).into_response();
    }
    let weeks = req.weeks_per_month.unwrap_or(4.33);
    let monthly_hours = req.hours_per_week * weeks;
    let weekly_gross = req.hourly_rate * req.hours_per_week;
    let monthly_gross = req.hourly_rate * monthly_hours;
    (
        StatusCode::OK,
        Json(CalcResponse {
            monthly_hours: round2(monthly_hours),
            monthly_gross: round2(monthly_gross),
            weekly_gross: round2(weekly_gross),
        }),
    )
        .into_response()
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/api/calc", post(calc))
        .layer(TraceLayer::new_for_http());

    let addr_str = std::env::var("TEMPS_ADDR")
        .or_else(|_| std::env::var("PORT"))
        .unwrap_or_else(|_| ":8080".to_string());
    // Accept both ":8080" and "0.0.0.0:8080" forms.
    let addr: SocketAddr = if addr_str.starts_with(':') {
        format!("0.0.0.0{}", addr_str).parse().expect("adresse invalide")
    } else {
        addr_str.parse().expect("adresse invalide")
    };

    tracing::info!("tempspartiel en écoute sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_ok() {
        let app = Router::new().route("/healthz", get(healthz));
        let res = app
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}