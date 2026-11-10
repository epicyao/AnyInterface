use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::process::ExitCode;

pub mod utils;

use utils::data_trans;

#[derive(Serialize)]
struct PingData {
    message: &'static str,
}

async fn ping() -> Json<utils::Response<PingData>> {
    Json(data_trans(true, 200, PingData { message: "pong" }))
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(error) = result {
                    eprintln!("failed to listen for Ctrl+C: {error}");
                }
            }
            _ = terminate.recv() => {}
        }
    }

    #[cfg(windows)]
    {
        use tokio::signal::windows::{ctrl_break, ctrl_close, ctrl_logoff, ctrl_shutdown};

        let mut ctrl_break = ctrl_break().expect("failed to install CTRL+BREAK handler");
        let mut ctrl_close = ctrl_close().expect("failed to install CTRL+CLOSE handler");
        let mut ctrl_logoff = ctrl_logoff().expect("failed to install CTRL+LOGOFF handler");
        let mut ctrl_shutdown = ctrl_shutdown().expect("failed to install CTRL+SHUTDOWN handler");

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                if let Err(error) = result {
                    eprintln!("failed to listen for Ctrl+C: {error}");
                }
            }
            _ = ctrl_break.recv() => {}
            _ = ctrl_close.recv() => {}
            _ = ctrl_logoff.recv() => {}
            _ = ctrl_shutdown.recv() => {}
        }
    }

    #[cfg(not(any(unix, windows)))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to listen for Ctrl+C: {error}");
    }

    println!("Shutdown signal received; finishing active requests...");
}

fn app() -> Router {
    Router::new().route("/v1/ping", get(ping))
}

#[tokio::main]
async fn main() -> ExitCode {
    let address = "localhost:3000";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| {
            eprintln!("failed to bind HTTP listener: {error}");
            error
        });

    let listener = match listener {
        Ok(listener) => listener,
        Err(_) => return ExitCode::FAILURE,
    };

    println!("HTTP server listening on http://{address}");
    if let Err(error) = axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("HTTP server failed: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::app;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn ping_returns_pong_response() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/v1/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "ok": true,
                "code": 200,
                "data": { "message": "pong" }
            })
        );
    }
}
