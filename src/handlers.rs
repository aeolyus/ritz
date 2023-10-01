pub mod asset;
pub mod commit;
pub mod log;
pub mod refs;

use crate::config::Config;
use crate::util::xmlencode;
use axum::{extract::Path, http::header, response::Html};
use git2::{ObjectType, Repository, Tree};

pub async fn root() -> Html<String> {
    let config = Config::load();
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

pub async fn tree(Path((repo, path)): Path<(String, String)>) -> Html<String> {
    let mut result: Vec<String> = Vec::new();
    let config = Config::load();
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

    let repo =
        Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();
    let head = repo.revparse_single("HEAD").unwrap();
    let path = std::path::Path::new(&path).strip_prefix("/").unwrap();
    let head_commit = head.into_commit().unwrap();
    let head_tree = head_commit.tree().unwrap();
    let obj = if !path.eq(std::path::Path::new("")) {
        head_tree.get_path(path).unwrap().to_object(&repo).unwrap()
    } else {
        head_tree.as_object().to_owned()
    };
    match obj.kind().unwrap() {
        ObjectType::Tree => {
            let tree = obj.into_tree().unwrap();
            result.append(&mut write_files(&repo, &tree));
        }
        ObjectType::Blob => {
            let filename = basename(path.to_str().unwrap(), '/');
            let blob = obj.into_blob().unwrap();
            result.push(format!("<p>{} ({}B)</p>", filename, blob.size()));
            result.push("<hr>".to_string());
            if blob.is_binary() {
                result.push("<p>Binary file.</p>".to_string());
            } else {
                result.push(format!(
                    "<pre>{}</pre>",
                    xmlencode(
                        std::str::from_utf8(blob.content())
                            .unwrap()
                            .to_string()
                            .as_ref()
                    )
                ));
            }
        }
        _ => (),
    };
    result.push(footer().to_string());
    Html(result.join(""))
}

fn header() -> String {
    format!(
        "<!DOCTYPE html><html> \
  <head> \
  <link rel=\"stylesheet\" type=\"text/css\" href=\"/static/style.css\" />
  <link rel=\"icon\" type=\"image/x-icon\" href=\"/static/favicon.ico\">
  </head> \
  <body>"
    )
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

fn write_files(repo: &Repository, tree: &Tree) -> Vec<String> {
    let mut result = Vec::new();
    result.push("<table>".to_string());
    result.push(
        "<thead><tr>
        <td><b>Mode</b></td>
        <td><b>Name</b></td>
        <td><b>Size</b></td>
        </tr></thread>"
            .to_string(),
    );
    for te in tree.iter() {
        result.push("<tr>".to_string());
        result.push(format!("<td>{:o}</td>", te.filemode()));
        result.push(format!(
            "<td><a href={}/>{}</a></td>",
            te.name().unwrap().to_string(),
            te.name().unwrap().to_string(),
        ));
        let obj = te.to_object(repo).unwrap();
        match obj.kind().unwrap() {
            ObjectType::Blob => {
                let blob = obj.into_blob().unwrap();
                result.push(format!("<td>{}</td>", blob.size()));
            }
            ObjectType::Tree => {
                result.push(format!("<td>{}</td>", 0));
            }
            _ => (),
        }
        result.push("</tr>".to_string());
    }
    result.push("</table>".to_string());
    result
}
