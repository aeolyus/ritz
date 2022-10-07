use axum::{extract::Path, response::Html, routing::get, Router};
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{Oid, Repository};
use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

const STD_PORT: u16 = 3000;

struct Config {
    dir: String,
    port: u16,
}

impl Config {
    fn load() -> Self {
        let dir = env::var("RITZ_DIR").unwrap_or("./".to_string());
        let port = env::var("RITZ_PORT")
            .unwrap_or(STD_PORT.to_string())
            .parse::<u16>()
            .unwrap();
        Config { dir, port }
    }
}

#[tokio::main]
async fn main() {
    let config: Config = Config::load();
    let app = Router::new()
        .route("/", get(root))
        .route("/:repo", get(log))
        .route("/:repo/commit/:hash", get(commit))
        .route("/:repo/log", get(log))
        .route("/:repo/refs", get(refs))
        .route("/:repo/tree/*path", get(tree))
        .route("/favicon.ico", get(favicon_handler));

    let sock_addr = SocketAddr::from((IpAddr::V6(Ipv6Addr::LOCALHOST), config.port));
    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<String> {
    let config: Config = Config::load();
    let mut result: Vec<String> = Vec::new();
    result.push(header().to_string());
    result.push("<span>Repositories</span>".to_string());
    result.push("<hr/>".to_string());
    let mut paths = std::fs::read_dir(config.dir)
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

async fn log(Path(repo): Path<String>) -> Html<String> {
    let config: Config = Config::load();
    let mut result: Vec<String> = Vec::new();
    let baseurl = repo.to_string();
    result.push(header().to_string());
    result.push(format!("<h1>{repo}</h1>"));
    result.push(format!("<span>git clone git://{repo}.git</span>"));
    result.push(format!(
        "<span>
    <a href=\"/{baseurl}/log\">Log</a>
    <a href=\"/{baseurl}/tree\">Tree</a>
    <a href=\"/{baseurl}/refs\">Refs</a>
            </span>"
    ));
    result.push("<hr/>".to_string());

    result.push("<table>".to_string());
    result.push(
        "<thead><tr>
        <td><b>Date</b></td>
        <td><b>Commit message</b></td>
        <td><b>Author</b></td>
        <td><b>Files</b></td>
        <td><b>+</b></td>
        <td><b>-</b></td>
        </tr></thread>"
            .to_string(),
    );

    let repo = Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    for rev in revwalk {
        let commit = repo.find_commit(rev.unwrap()).unwrap();
        let message = commit.summary_bytes().unwrap_or(commit.message_bytes());
        result.push("<tr>".to_string());
        let naive = NaiveDateTime::from_timestamp(commit.time().seconds(), 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let formatted_datetime = datetime.format("%Y-%m-%d %H:%M");
        result.push(format!("<td>{}</td>", formatted_datetime));
        result.push(format!(
            "<td><a href=\"/{baseurl}/commit/{}\">{}</a></td>",
            commit.id(),
            String::from_utf8_lossy(message)
        ));
        result.push(format!("<td>{}</td>", commit.author().name().unwrap()));
        let tree = &Some(commit.tree().unwrap());
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0).unwrap().tree().unwrap())
        } else {
            None
        };
        let diff = Repository::diff_tree_to_tree(&repo, parent_tree.as_ref(), tree.as_ref(), None)
            .unwrap();
        let diffstats = diff.stats().unwrap();
        result.push(format!("<td>{}</td>", diffstats.files_changed()));
        result.push(format!("<td>+{}</td>", diffstats.insertions()));
        result.push(format!("<td>-{}</td>", diffstats.deletions()));
        result.push("</tr>".to_string());
    }

    result.push("</table>".to_string());
    result.push(footer().to_string());
    Html(result.join(""))
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
    let mut result: Vec<String> = Vec::new();
    let config = Config::load();
    result.push(header().to_string());
    result.push(format!("<h1>{repo}</h1>"));
    result.push(format!("<span>git clone git://{repo}.git</span>"));
    result.push(format!(
        "<span>
    <a href=\"/{repo}/log\">Log</a>
    <a href=\"/{repo}/tree\">Tree</a>
    <a href=\"/{repo}/refs\">Refs</a>
            </span>"
    ));
    result.push("<hr/>".to_string());

    let repo = Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();
    result.push("<pre>".to_string());
    result.push("<b>commit</b> ".to_string());
    result.push(format!("<a href=\"../commit/{hash}\">{hash}</a>\n"));

    let commit = repo.find_commit(Oid::from_str(&hash).unwrap()).unwrap();
    if commit.parent_count() > 0 {
        let parent_hash = commit.parent(0).unwrap().id();
        result.push("<b>parent</b> ".to_string());
        result.push(format!(
            "<a href=\"../commit/{parent_hash}\">{parent_hash}</a>\n"
        ));
    }

    let author = commit.author().name().unwrap().to_string();
    let email = commit.author().email().unwrap().to_string();
    result.push(format!(
        "<b>Author:</b> {author} <<a href=\"mailto:{email}\">{email}</a>>\n"
    ));
    result.push("<b>Date:</b>   ".to_string());
    let naive = NaiveDateTime::from_timestamp(commit.time().seconds(), 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let formatted_datetime = datetime.format("%a, %Y %b %e %H:%M:%S");
    let date_offset = commit.time().offset_minutes();
    if date_offset < 0 {
        result.push(format!(
            "{formatted_datetime} -{:02}{:02}\n",
            -date_offset / 60,
            -date_offset % 60
        ));
    } else {
        result.push(format!(
            "{formatted_datetime} +{:02}{:02}\n",
            date_offset / 60,
            date_offset % 60
        ));
    }

    match commit.message() {
        Some(m) => result.push(format!("\n{}\n", m.to_string())),
        _ => (),
    }
    result.push("</pre>".to_string());
    result.push(footer().to_string());
    Html(result.join(""))
}

// TODO: Handle favicon more gracefully
async fn favicon_handler() -> &'static str {
    r"This is where I'd put my favicon if I had one ¯\_(ツ)_/¯"
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
