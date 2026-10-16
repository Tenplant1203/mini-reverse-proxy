use axum::{
    Router,
    extract::Request,
    http::{self, Response},
    routing::get,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/debug", get(debug).post(debug));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}

async fn debug(request: Request) -> axum::http::Response<String> {
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(err) => return Response::new(format!("to bytes error: {err}")),
    };
    let body_text = String::from_utf8_lossy(&body_bytes);
    let mut response = Response::new(format!(
        "method: {}\nuri: {}\nheaders: {:?}\nbody: {}\n",
        parts.method, parts.uri, parts.headers, body_text
    ));
    response.headers_mut().insert(
        http::header::CONNECTION,
        http::HeaderValue::from_static("x-upstream-test"),
    );
    response.headers_mut().insert(
        http::header::HeaderName::from_static("x-upstream-test"),
        http::HeaderValue::from_static("hello"),
    );
    response
}
