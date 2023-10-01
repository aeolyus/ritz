mod config;
mod data;
mod error;
mod handlers;
mod util;
use axum::{routing::get, Router};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

#[tokio::main]
async fn main() {
    let conf = config::Config::load();
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/static/*path", get(handlers::asset::asset))
        .route("/:repo", get(handlers::log::log))
        .route("/:repo/commit/:hash", get(handlers::commit::commit))
        .route("/:repo/log", get(handlers::log::log))
        .route("/:repo/refs", get(handlers::refs::refs))
        .route("/:repo/tree/*path", get(handlers::tree));
    let sock_addr =
        SocketAddr::from((IpAddr::V6(Ipv6Addr::LOCALHOST), conf.port));
    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
