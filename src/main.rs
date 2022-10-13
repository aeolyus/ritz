mod config;
mod handlers;
use axum::{routing::get, Router};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

#[tokio::main]
async fn main() {
    let conf = config::Config::load();
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/:repo", get(handlers::log))
        .route("/:repo/commit/:hash", get(handlers::commit))
        .route("/:repo/log", get(handlers::log))
        .route("/:repo/refs", get(handlers::refs))
        .route("/:repo/tree/*path", get(handlers::tree))
        .route("/favicon.ico", get(handlers::favicon_handler));

    let sock_addr = SocketAddr::from((IpAddr::V6(Ipv6Addr::LOCALHOST), conf.port));
    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
