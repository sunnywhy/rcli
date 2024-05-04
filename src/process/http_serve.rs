use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on port {}", &path, addr);
    let state = HttpServeState { path: path.clone() };

    // axum router
    let router = Router::new()
        .route("/*path", get(file_handler))
        .nest_service("/tower", ServeDir::new(path))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        return (StatusCode::NOT_FOUND, format!("File not found: {:?}", p));
    }

    if p.is_dir() {
        return match process_dir(&p).await {
            Ok(content) => {
                info!("Read {} bytes", content.len());
                (StatusCode::OK, content)
            }
            Err(e) => {
                warn!("Error reading directory: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error reading directory: {:?}", e),
                )
            }
        };
    }

    match tokio::fs::read_to_string(p).await {
        Ok(content) => {
            info!("Read {} bytes", content.len());
            (StatusCode::OK, content)
        }
        Err(e) => {
            warn!("Error reading file: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error reading file: {:?}", e),
            )
        }
    }
}

async fn process_dir(p: &std::path::Path) -> anyhow::Result<String> {
    let mut content = String::new();
    content.push_str("<html><head><title>Directory listing</title></head><body>");
    content.push_str("<ul>");
    let mut entries = tokio::fs::read_dir(p).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let name = entry.file_name();
        content.push_str("<li>");
        content.push_str("<a href=\"");
        content.push_str(&path.to_string_lossy());
        content.push_str("\">");
        content.push_str(&name.to_string_lossy());
        content.push_str("</a>");
        content.push_str("</li>");
    }

    content.push_str("</ul>");
    content.push_str("</body></html>");
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = HttpServeState {
            path: PathBuf::from("."),
        };
        let state = Arc::new(state);
        let (status, content) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        assert!(content.contains("[package]"));
    }
}
