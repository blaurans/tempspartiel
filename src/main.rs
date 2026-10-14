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
    full_time_net: f64,
    percent: f64,
    months: u32,
    phase2_rate: Option<f64>,
}

#[derive(Serialize, Clone)]
struct MonthRow {
    month: u32,
    worked_net: f64,
    ij_net: f64,
    complement: f64,
    total_net: f64,
    gap: f64,
    cumulative_gap: f64,
    ij_exhausted: bool,
    phase: &'static str,
}

#[derive(Serialize)]
struct CalcResponse {
    full_time_net: f64,
    percent: f64,
    phase2_rate: f64,
    months: Vec<MonthRow>,
    total_income: f64,
    total_gap: f64,
    full_time_total: f64,
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

async fn calc(Json(req): Json<CalcRequest>) -> impl IntoResponse {
    if req.full_time_net <= 0.0 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"salaire invalide"}))).into_response();
    }
    if !(10.0..=90.0).contains(&req.percent) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"pourcentage invalide (10 a 90)"}))).into_response();
    }
    if req.months == 0 || req.months > 36 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"duree invalide (1 a 36 mois)"}))).into_response();
    }
    let phase2_rate = req.phase2_rate.unwrap_or(0.97).clamp(0.5, 1.0);

    // Estimation net/brut : charges salariales ~22% (coef. approx.).
    let gross_full = req.full_time_net / 0.78;
    let p = req.percent / 100.0;
    let worked_share = p;
    let unworked_share = 1.0 - p;

    let annual_ij_cap_day = 41.95_f64;
    let mut cum_unworked_days = 0.0_f64;
    let mut cumulative_gap = 0.0_f64;
    let mut total_income = 0.0_f64;
    let mut rows = Vec::with_capacity(req.months as usize);

    for m in 1..=req.months {
        let phase = if m <= 6 { "Phase 1 (100% net)" } else { "Phase 2 (95-99% net)" };
        let phase_rate = if m <= 6 { 1.0 } else { phase2_rate };

        let days_unworked_this_month = 30.0 * unworked_share;
        let remaining_ij_days = (360.0 - cum_unworked_days).max(0.0);
        let days_paid_ij = days_unworked_this_month.min(remaining_ij_days);
        let exhausted = remaining_ij_days <= 0.0;

        cum_unworked_days += days_unworked_this_month;

        // IJSS bruts (estimation).
        let ij_daily_base = gross_full / 30.0;
        let ij_daily = (0.5 * ij_daily_base).min(annual_ij_cap_day);
        let ij_gross = if exhausted { 0.0 } else { ij_daily * days_paid_ij };
        // CGS/RDS sur IJ ~ 6,7 %.
        let ij_net = ij_gross * (1.0 - 0.067);

        let worked_net = req.full_time_net * worked_share;
        let target_net = req.full_time_net * phase_rate;

        let mut complement = (target_net - worked_net - ij_net).max(0.0);
        let mut total_net = worked_net + ij_net + complement;
        // Plafonnement légal : cumul <= salaire habituel.
        if total_net > req.full_time_net {
            total_net = req.full_time_net;
            complement = (req.full_time_net - worked_net - ij_net).max(0.0);
        }
        // Si droits IJ épuisés, pas de maintien complementaire au-delà du travaillé.
        if exhausted {
            complement = 0.0;
            total_net = worked_net + ij_net;
            if total_net > req.full_time_net {
                total_net = req.full_time_net;
            }
        }

        let gap = total_net - req.full_time_net;
        cumulative_gap += gap;
        total_income += total_net;

        rows.push(MonthRow {
            month: m,
            worked_net: round2(worked_net),
            ij_net: round2(ij_net),
            complement: round2(complement),
            total_net: round2(total_net),
            gap: round2(gap),
            cumulative_gap: round2(cumulative_gap),
            ij_exhausted: exhausted,
            phase,
        });
    }

    let resp = CalcResponse {
        full_time_net: round2(req.full_time_net),
        percent: req.percent,
        phase2_rate,
        months: rows,
        total_income: round2(total_income),
        total_gap: round2(cumulative_gap),
        full_time_total: round2(req.full_time_net * req.months as f64),
    };
    (StatusCode::OK, Json(resp)).into_response()
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

    #[tokio::test]
    async fn calc_one_month() {
        let app = Router::new().route("/api/calc", post(calc));
        let body = serde_json::json!({"full_time_net":2000.0,"percent":50.0,"months":1});
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/calc")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}