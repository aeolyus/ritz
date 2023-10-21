use crate::config::Config;
use crate::data::{get_commitinfo, CommitInfo};
use crate::error::AppError;
use crate::handlers::{footer, header};
use crate::util::{print_time_short, xmlencode};
use anyhow::Result;
use axum::{extract::Path, response::Html};
use git2::{Reference, Repository};
use std::cmp::Ordering;
use std::fmt::Write;

struct ReferenceInfo<'a> {
    rf: Reference<'a>,
    commitinfo: CommitInfo<'a>,
}

pub async fn refs(Path(repo): Path<String>) -> Result<Html<String>, AppError> {
    let config = Config::load();
    let mut result = String::new();
    result.push_str(&header());
    result.push_str(&format!("<h1>{repo}</h1>"));
    result.push_str(&format!("<span>git clone git://{repo}.git</span>"));
    result.push_str(&format!(
        "<span>
    <a href=\"/{repo}/log\">Log</a>
    <a href=\"/{repo}/tree\">Tree</a>
    <a href=\"/{repo}/refs\">Refs</a>
            </span>"
    ));
    result.push_str("<hr/>");

    let repo =
        Repository::open(std::path::Path::new(&config.dir).join(repo)).unwrap();
    write_refs(&mut result, &repo)?;
    result.push_str(footer());
    Ok(Html(result))
}

fn write_refs<W: Write>(w: &mut W, repo: &Repository) -> Result<()> {
    let mut j = 0;
    let mut count = 0;
    let titles = vec!["Branches", "Tags"];
    let ids = vec!["branches", "tags"];
    let refs = get_refs(repo)?;
    for (_i, r) in refs.iter().enumerate() {
        if j == 0 && r.rf.is_tag() {
            if count >= 1 {
                write!(w, "</tbody></table><br/>\n")?;
            }
            count = 0;
            j = 1;
        }

        // Print header if it has an entry first
        if count == 0 {
            count += 1;
            write!(
                w,
                "<h2>{}</h2>
                   <table id=\"{}\">
                   <thead>\n<tr>
                   <td><b>Name</b></td>
                   <td><b>Last commit date</b></td>
                   <td><b>Author</b></td>
                   </tr></thead>
                   <tbody>",
                titles[j], ids[j]
            )?;
        }

        write!(w, "<tr><td>")?;
        write!(w, "{}", xmlencode(&r.rf.shorthand().unwrap_or("")))?;
        write!(w, "</td><td>")?;
        print_time_short(w, r.commitinfo.author.when())?;
        write!(w, "</td><td>")?;
        write!(
            w,
            "{}",
            xmlencode(&r.commitinfo.author.name().unwrap_or(""))
        )?;
        write!(w, "</td></tr>\n")?;
    }
    if count >= 1 {
        write!(w, "</tbody></table>")?;
    }
    Ok(())
}

/// Returns a [ReferenceInfo] vector of branches and tags sorted by [refs_cmp]
fn get_refs(repo: &Repository) -> Result<Vec<ReferenceInfo>> {
    let mut ris = repo
        .references()?
        .filter_map(|rf| rf.ok())
        .filter(|rf| rf.is_tag() | rf.is_branch())
        .filter_map(|rf| {
            let obj = rf.peel(git2::ObjectType::Any).ok()?;
            let commitinfo = get_commitinfo(repo, obj.id().to_string()).ok()?;
            Some(ReferenceInfo { rf, commitinfo })
        })
        .collect::<Vec<ReferenceInfo>>();
    ris.sort_by(refs_cmp);
    Ok(ris)
}

/// Sort by type with branch first, by date with most recent first, then
/// alphabetically by shorthand name
fn refs_cmp(a: &ReferenceInfo, b: &ReferenceInfo) -> Ordering {
    a.rf.is_tag()
        .cmp(&b.rf.is_tag())
        .then(b.commitinfo.author.when().cmp(&a.commitinfo.author.when()))
        .then(a.rf.shorthand_bytes().cmp(b.rf.shorthand_bytes()))
}
