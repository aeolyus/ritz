use crate::config::Config;
use crate::error::AppError;
use crate::handlers::{footer, header, print_diff_line};
use anyhow::Result;
use axum::{extract::Path, response::Html};
use chrono::{NaiveDateTime, TimeZone, Utc};
use git2::{
    Diff, DiffFormat, DiffStatsFormat, Oid, Repository, Signature, Time, Tree,
};
use std::fmt::Write;

pub async fn commit(
    Path((repo, hash)): Path<(String, String)>,
) -> Result<Html<String>, AppError> {
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
    let commit = repo.find_commit(Oid::from_str(&hash).unwrap()).unwrap();

    let mut temp_buf = String::new();
    let ci = &get_commitinfo(&repo, hash)?;
    print_commit(&mut temp_buf, ci)?;
    result.push(temp_buf);

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
    Ok(Html(result.join("")))
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

fn print_commit<W: Write>(w: &mut W, ci: &CommitInfo) -> Result<()> {
    write!(w, "<b>commit</b> ")?;
    write!(w, "<a href=\"../commit/{}\">{}</a>\n", ci.oid, ci.oid)?;
    if let Some(poid) = &ci.parentoid {
        write!(w, "<b>parent</b> ")?;
        write!(w, "<a href=\"../commit/{}\">{}</a>\n", poid, poid)?;
    }
    write!(w, "<b>Author:</b> ")?;
    write!(w, "{}", ci.author.name().unwrap_or(""))?;
    let email = ci.author.email().unwrap_or("");
    write!(w, " <<a href=\"mailto:{}]\">{}</a>>\n", email, email)?;
    write!(w, "<b>Date:</b>   ")?;
    print_time(w, ci.author.when())?;
    write!(w, "\n")?;
    if let Some(msg) = &ci.msg {
        write!(w, "\n{}\n", msg)?;
    }
    return Ok(());
}

fn print_time<W: Write>(w: &mut W, intime: Time) -> Result<()> {
    let utc = &NaiveDateTime::from_timestamp(
        intime.seconds() + i64::from(intime.offset_minutes() * 60),
        0,
    );
    let dt = Utc.from_utc_datetime(utc);
    let fmt_dt = dt.format("%a, %Y %b %e %H:%M:%S %:z");
    write!(w, "{}", fmt_dt)?;
    return Ok(());
}
