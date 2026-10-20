use axum::{
    routing::get,
    response::Json,
    Router,
    http::Uri,
};
use axum::body::Body;
use axum::response::Response;
use tower_http::{
    services::ServeDir,
    cors::{Any, CorsLayer},
};
use shared::get_stations;
use reqwest::Client;

async fn get_stations_handler() -> Json<serde_json::Value> {
    let stations = get_stations();
    Json(serde_json::to_value(stations).unwrap())
}

// Proxy endpoint for radio streams - makes cross-origin streams same-origin for Web Audio API
async fn stream_proxy(uri: Uri) -> Result<Response<Body>, (axum::http::StatusCode, String)> {
    let query = uri.query().ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "Missing 'url' query parameter".to_string(),
    ))?;

    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

    let stream_url = params.get("url").ok_or((
        axum::http::StatusCode::BAD_REQUEST,
        "Missing 'url' query parameter".to_string(),
    ))?;

    // Fetch the stream
    let client = Client::new();
    let response = client.get(stream_url.as_str())
        .header("User-Agent", "Mozilla/5.0 (compatible; RadioPlayer/1.0)")
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_GATEWAY, e.to_string()))?;

    let status = response.status();
    let content_type = response.headers().get("content-type").cloned();

    // Build response with streaming body
    let body = Body::from_stream(response.bytes_stream());

    let mut response = Response::new(body);
    *response.status_mut() = status;

    // Forward content-type header
    if let Some(ct) = content_type {
        response.headers_mut().insert("content-type", ct);
    }

    Ok(response)
}

#[tokio::main]
async fn main() {
    let dist_dir = ServeDir::new("frontend/dist");

    // Build the application router
    let app = Router::new()
        .route("/api/stations", get(get_stations_handler))
        .route("/api/stream", get(stream_proxy))
        .fallback_service(dist_dir)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("📻 Radio player server running at http://0.0.0.0:3001");
    println!("📡 API endpoint: http://0.0.0.0:3001/api/stations");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
