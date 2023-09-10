use axum::{
    body::{self, Empty, Full},
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

static STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

pub async fn asset(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches("/");
    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap(),
        Some(f) => Response::builder()
            .status(StatusCode::OK)
            .body(body::boxed(Full::from(f.contents())))
            .unwrap(),
    }
}
