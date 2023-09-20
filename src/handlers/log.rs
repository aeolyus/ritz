use crate::config::Config;
use crate::data::{self, CommitInfo};
use crate::error::AppError;
use crate::handlers::{footer, header};
use crate::util::print_time_short;
use anyhow::{anyhow, Result};
use axum::{extract::Path, response::Html};
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{Oid, Repository};
use std::fmt::Write;

pub async fn log(Path(repo): Path<String>) -> Result<Html<String>, AppError> {
    let config = Config::load();
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

    result.push("<table id=\"log\">".to_string());
    result.push(
        "<thead><tr>
        <td><b>Date</b></td>
        <td><b>Commit message</b></td>
        <td><b>Author</b></td>
        <td><b>Files</b></td>
        <td align=\"right\"><b>+</b></td>
        <td align=\"right\"><b>-</b></td>
        </tr></thread>"
            .to_string(),
    );

    let repo =
        Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();

    let oid = repo
        .head()?
        .target()
        .ok_or(anyhow!("No Oid for the current repo HEAD"))?;
    let mut buf = String::new();
    print_log(&mut buf, baseurl.as_ref(), &repo, oid)?;
    result.push(buf);

    Ok(Html(result.join("")))
}

fn print_log_line<W: Write>(
    w: &mut W,
    relpath: &str,
    ci: &CommitInfo,
) -> Result<()> {
    write!(w, "<tr><td>")?;
    print_time_short(w, ci.author.when())?;
    write!(w, "</td><td>")?;
    if let Some(summary) = &ci.summary {
        write!(w, "<a href=\"/{}/commit/{}/\">", relpath, ci.oid)?;
        write!(w, "{}", summary)?;
        write!(w, "</a>")?;
    }
    write!(w, "</td><td>")?;
    write!(w, "{}", ci.author.name().unwrap_or(""))?;
    write!(w, "</td><td class=\"num\" align=\"right\">")?;
    write!(w, "{}", ci.file_count)?;
    write!(w, "</td><td class=\"num\" align=\"right\">")?;
    write!(w, "+{}", ci.add_count)?;
    write!(w, "</td><td class=\"num\" align=\"right\">")?;
    write!(w, "-{}", ci.del_count)?;
    write!(w, "</td></tr>\n")?;
    Ok(())
}

fn print_log<W: Write>(
    w: &mut W,
    relpath: &str,
    repo: &Repository,
    oid: Oid,
) -> Result<()> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push(oid)?;
    for id in revwalk {
        if id.is_err() {
            break;
        }
        let id = id.unwrap();
        let ci = data::get_commitinfo(repo, id.to_string())?;
        print_log_line(w, relpath, &ci)?;
    }
    Ok(())
}
