use crate::actuators::{ActuatorDriver, LocalActuatorDriver};
use axum::extract::State;
use axum::http::header::{HeaderName, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    api_key: String,
    driver: Arc<Mutex<Box<dyn ActuatorDriver>>>,
}

#[derive(Deserialize)]
struct FeederRequest {
    device_key: String,
    duration_ms: Option<u64>,
}

#[derive(Deserialize)]
struct DoorRequest {
    device_key: String,
}

#[derive(Serialize)]
struct ApiResponse {
    status: &'static str,
    message: String,
}

fn authorized(headers: &HeaderMap, expected_key: &str) -> bool {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected_key)
        .unwrap_or(false)
}

pub async fn run_actuator_server(
    bind_addr: &str,
    api_key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        api_key,
        driver: Arc::new(Mutex::new(Box::new(LocalActuatorDriver::default()))),
    };

    let app = Router::new()
        .route("/actuators/feeder/activate", post(feeder_activate))
        .route("/actuators/door/open", post(door_open))
        .route("/actuators/door/close", post(door_close))
        .with_state(state)
        .layer(cors_layer());

    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn cors_layer() -> CorsLayer {
    let x_api_key = HeaderName::from_static("x-api-key");
    let base = CorsLayer::new()
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, x_api_key]);

    match env::var("ACTUATOR_ALLOWED_ORIGIN") {
        Ok(origin) if !origin.is_empty() && origin != "*" => {
            if let Ok(value) = HeaderValue::from_str(&origin) {
                base.allow_origin(value)
            } else {
                base.allow_origin(Any)
            }
        }
        _ => base.allow_origin(Any),
    }
}

async fn feeder_activate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FeederRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    if !authorized(&headers, &state.api_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                status: "error",
                message: "unauthorized".to_string(),
            }),
        );
    }

    let mut driver = state.driver.lock().await;
    match driver.feeder_activate(&payload.device_key, payload.duration_ms.unwrap_or(2500)) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: "ok",
                message: "feeder activated".to_string(),
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                status: "error",
                message: err,
            }),
        ),
    }
}

async fn door_open(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DoorRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    if !authorized(&headers, &state.api_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                status: "error",
                message: "unauthorized".to_string(),
            }),
        );
    }

    let mut driver = state.driver.lock().await;
    match driver.door_open(&payload.device_key) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: "ok",
                message: "door opened".to_string(),
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                status: "error",
                message: err,
            }),
        ),
    }
}

async fn door_close(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DoorRequest>,
) -> (StatusCode, Json<ApiResponse>) {
    if !authorized(&headers, &state.api_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                status: "error",
                message: "unauthorized".to_string(),
            }),
        );
    }

    let mut driver = state.driver.lock().await;
    match driver.door_close(&payload.device_key) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                status: "ok",
                message: "door closed".to_string(),
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                status: "error",
                message: err,
            }),
        ),
    }
}
