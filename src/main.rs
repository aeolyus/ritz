use axum::{extract::Path, response::Html, routing::get, Router};
use git2::Repository;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/:repo", get(repo))
        .route("/:repo/commit/:hash", get(commit))
        .route("/:repo/log", get(log))
        .route("/:repo/refs", get(refs))
        .route("/:repo/tree/*path", get(tree));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<String> {
    let mut result: Vec<String> = Vec::new();
    result.push(header().to_string());
    result.push("<span>Repositories</span>".to_string());
    result.push("<hr/>".to_string());
    let mut paths = std::fs::read_dir("./")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| Repository::open(path).is_ok())
        .map(|path| path.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();
    paths.sort();
    result.push("<table>".to_string());
    result.push("<thead><tr><td><b>Name</b></td></tr></thread>".to_string());
    for path in paths {
        let repo = basename(&path, '/');
        result.push("<tr><td>".to_string());
        result.push(format!("<a href=/{}>{}</a>", repo, repo));
        result.push("</td></td>".to_string());
    }
    result.push("</table>".to_string());
    result.push(footer().to_string());
    Html(result.join(""))
}

async fn repo(Path(repo): Path<String>) -> Html<String> {
    Html(format!(
        "{}\n\
        <h1>[wip] repo</h1>\n\
        <h1>Repository: {repo}</h1>\n\
        {}",
        header(),
        footer()
    ))
}

async fn log(Path(repo): Path<String>) -> Html<String> {
    Html(format!(
        "{}\n\
        <h1>[wip] log</h1>\n\
        <h1>Repository: {repo}</h1>\n\
        {}",
        header(),
        footer()
    ))
}

async fn refs(Path(repo): Path<String>) -> Html<String> {
    Html(format!(
        "{}\n\
        <h1>[wip] refs</h1>\n\
        <h1>Repository: {repo}</h1>\n\
        {}",
        header(),
        footer()
    ))
}

async fn tree(Path((repo, path)): Path<(String, String)>) -> Html<String> {
    Html(format!(
        "{}\n\
        <h1>[wip] tree</h1>\n\
        <h1>Repository: {repo}</h1>\n\
        <h1>Path: {path}</h1>\n\
        {}",
        header(),
        footer()
    ))
}

async fn commit(Path((repo, hash)): Path<(String, String)>) -> Html<String> {
    Html(format!(
        "{}\n\
        <h1>[wip] commit</h1>\n\
        <h1>Repository: {repo}</h1>\n\
        <h1>Hash: {hash}</h1>\n\
        {}",
        header(),
        footer()
    ))
}

fn header() -> &'static str {
    "<!DOCTYPE html><html><body>"
}

fn footer() -> &'static str {
    "</body></html>"
}
fn basename(path: &str, sep: char) -> &str {
    let mut pieces = path.rsplit(sep);
    match pieces.next() {
        Some(p) => p.into(),
        None => path.into(),
    }
}
