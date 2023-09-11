use crate::config::Config;
use crate::handlers::{footer, header, print_diff_line};
use anyhow::Result;
use axum::{extract::Path, response::Html};
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{
    Diff, DiffFormat, DiffStatsFormat, Oid, Repository, Signature, Tree,
};

pub async fn commit(
    Path((repo, hash)): Path<(String, String)>,
) -> Html<String> {
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

    let repo =
        Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();
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

    let tree = &Some(commit.tree().unwrap());
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0).unwrap().tree().unwrap())
    } else {
        None
    };
    let diff = Repository::diff_tree_to_tree(
        &repo,
        parent_tree.as_ref(),
        tree.as_ref(),
        None,
    )
    .unwrap();
    let diffstats = diff.stats().unwrap();

    result.push("<b>Diffstat:</b>\n".to_string());
    result.push(
        diffstats
            .to_buf(DiffStatsFormat::FULL, 80)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    );
    result.push("<table>".to_string());
    result.push("</table>".to_string());

    result.push("</pre>".to_string());

    result.push("<pre>".to_string());
    result.push("<hr/>".to_string());

    diff.print(DiffFormat::Patch, |d, h, l| {
        print_diff_line(d, h, l, &mut result)
    })
    .unwrap();
    result.push("</pre>".to_string());

    result.push(footer().to_string());
    Html(result.join(""))
}

#[allow(dead_code)]
struct CommitInfo<'a> {
    oid: String,
    parentoid: Option<String>,
    author: Signature<'a>,
    msg: Option<String>,
    commit_tree: Tree<'a>,
    parent_tree: Option<Tree<'a>>,
    diff: Diff<'a>,
}

#[allow(dead_code)]
fn get_commitinfo(repo: &Repository, oid: String) -> Result<CommitInfo> {
    let commit = repo.find_commit(Oid::from_str(&oid)?)?;
    let parent = commit.parent(0).ok();
    let parentoid = parent.as_ref().map(|c| c.id().to_string());
    let author = commit.author().to_owned();
    let msg = commit.message().map(|s| s.into());
    let commit_tree = commit.tree()?;
    let parent_tree = parent.map(|c| c.tree().ok()).flatten();
    let diff = Repository::diff_tree_to_tree(
        repo,
        parent_tree.as_ref(),
        Some(&commit_tree),
        None,
    )?;
    Ok(CommitInfo {
        oid,
        parentoid,
        author,
        msg,
        commit_tree,
        parent_tree,
        diff,
    })
}
