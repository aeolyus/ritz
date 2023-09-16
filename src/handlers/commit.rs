use crate::config::Config;
use crate::error::AppError;
use crate::handlers::{footer, header};
use anyhow::{anyhow, Result};
use axum::{extract::Path, response::Html};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use git2::{
    Delta, Diff, DiffFindOptions, DiffFlags, Oid, Patch, Repository, Signature,
    Time, Tree,
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
    let mut buf = String::new();
    let ci = &get_commitinfo(&repo, hash)?;
    print_commit(&mut buf, ci)?;
    print_diffstat(&mut buf, ci)?;
    print_diff(&mut buf, ci)?;
    result.push(buf);
    result.push("</pre>".to_string());
    result.push(footer().to_string());
    Ok(Html(result.join("")))
}

struct DeltaInfo<'a> {
    #[allow(dead_code)]
    patch: Patch<'a>,
    add_count: usize,
    del_count: usize,
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
    deltas: Vec<DeltaInfo<'a>>,
    add_count: usize,
    del_count: usize,
    file_count: usize,
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
    let mut diff = Repository::diff_tree_to_tree(
        repo,
        parent_tree.as_ref(),
        Some(&commit_tree),
        None,
    )?;

    // Diff stats
    let mut add_count = 0;
    let mut del_count = 0;
    let file_count = diff.deltas().len();
    let mut opts = &mut DiffFindOptions::new();
    // Find exact match renames and copies
    opts = opts.renames(true).copies(true).exact_match_only(true);
    diff.find_similar(Some(&mut opts))?;
    let mut deltas = vec![];
    for (idx, _) in diff.deltas().enumerate() {
        let patch = Patch::from_diff(&diff, idx)?
            .ok_or(anyhow!("Error getting patch"))?;
        let (_, add, del) = patch.line_stats()?;
        let di = DeltaInfo {
            patch,
            add_count: add,
            del_count: del,
        };
        add_count += add;
        del_count += del;
        deltas.push(di);
    }

    Ok(CommitInfo {
        oid,
        parentoid,
        author,
        msg,
        commit_tree,
        parent_tree,
        diff,
        deltas,
        add_count,
        del_count,
        file_count,
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
    let utc = NaiveDateTime::from_timestamp_opt(intime.seconds(), 0)
        .ok_or(anyhow!("Error parsing timestamp seconds: {:#?}", intime))?;
    let offset = FixedOffset::east_opt(intime.offset_minutes() * 60).ok_or(
        anyhow!("Error parsing timestamp offset minutes: {:#?}", intime),
    )?;
    let dt: DateTime<FixedOffset> =
        DateTime::from_naive_utc_and_offset(utc, offset);
    let fmt_dt = dt.format("%a, %Y %b %e %H:%M:%S %:z");
    write!(w, "{}", fmt_dt)?;
    return Ok(());
}

fn print_diffstat<W: Write>(w: &mut W, ci: &CommitInfo) -> Result<()> {
    write!(w, "<b>Diffstat:</b>\n")?;
    write!(w, "<table>")?;
    const TOTAL: usize = 80;

    for (i, delta) in ci.diff.deltas().enumerate() {
        let c = match delta.status() {
            Delta::Added => 'A',
            Delta::Copied => 'C',
            Delta::Deleted => 'D',
            Delta::Modified => 'M',
            Delta::Renamed => 'R',
            Delta::Typechange => 'T',
            _ => ' ',
        };
        if c == ' ' {
            write!(w, "<tr><td>{}", c)?;
        } else {
            write!(w, "<tr><td class=\"{}\">{}", c, c)?;
        }
        write!(w, "</td><td><a href=\"#h{}\">", i)?;
        write!(
            w,
            "{}",
            delta
                .old_file()
                .path()
                .unwrap_or(std::path::Path::new(""))
                .display()
        )?;
        if delta.old_file().path() != delta.new_file().path() {
            write!(
                w,
                " -> {}",
                delta
                    .old_file()
                    .path()
                    .unwrap_or(std::path::Path::new(""))
                    .display()
            )?;
        }
        write!(w, "</a>")?;
        let mut add = ci.deltas[i].add_count;
        let mut del = ci.deltas[i].del_count;
        let changed = add + del;
        if changed > TOTAL {
            if add != 0 {
                add = (add / changed * TOTAL) + 1;
            }
            if del != 0 {
                del = (del / changed * TOTAL) + 1;
            }
        }
        write!(w, "</td><td> | </td>")?;
        write!(w, "<td class=\"num\">{}</td>", changed)?;
        write!(w, "<td><span class=\"i\">")?;
        write!(w, "{:+<1$}", "", add)?;
        write!(w, "</span><span class=\"d\">")?;
        write!(w, "{:-<1$}", "", del)?;
        write!(w, "</span></td></tr>\n")?;
    }
    write!(w, "</table></pre>")?;
    write!(
        w,
        "<pre>{} file{} changed, {} insertion{}(+), {} deletion{}(-)\n",
        ci.deltas.len(),
        match ci.deltas.len() {
            1 => "",
            _ => "s",
        },
        ci.add_count,
        match ci.add_count {
            1 => "",
            _ => "s",
        },
        ci.del_count,
        match ci.del_count {
            1 => "",
            _ => "s",
        },
    )?;
    write!(w, "<hr/>")?;
    Ok(())
}

fn print_diff<W: Write>(w: &mut W, ci: &CommitInfo) -> Result<()> {
    let diff = &ci.diff;
    for i in 0..diff.deltas().len() {
        let patch = Patch::from_diff(&diff, i)?
            .ok_or(anyhow!("Error getting patch"))?;
        let delta = patch.delta();

        let old_file = delta
            .old_file()
            .path()
            .unwrap_or(std::path::Path::new(""))
            .display();
        let new_file = delta
            .new_file()
            .path()
            .unwrap_or(std::path::Path::new(""))
            .display();
        write!(
            w,
            "<b>diff --git a/<a id=\"h{}\" href=\"../tree/{}\">{}</a>",
            i, old_file, old_file,
        )?;
        write!(
            w,
            " b/<a href=\"../tree/{}\">{}</a></b>\n",
            new_file, new_file
        )?;

        if delta.flags().contains(DiffFlags::BINARY) {
            write!(w, "Binary files differ\n")?;
        }

        for j in 0..patch.num_hunks() {
            let Ok((hunk, _)) = patch.hunk(j) else {
                break;
            };
            write!(
                w,
                "<a href=\"#h{}-{}\" id=\"h{}-{}\" class=\"h\">",
                i, j, i, j,
            )?;
            write!(w, "{}", String::from_utf8(hunk.header().to_vec())?)?;
            write!(w, "</a>")?;

            for k in 0..100 {
                let Ok(line) = patch.line_in_hunk(j, k) else {
                    break;
                };
                if line.old_lineno().is_none() {
                    write!(
                        w,
                        "<a href=\"#h{}-{}-{}\" id=\"h{}-{}-{}\" class=\"i\">+",
                        i, j, k, i, j, k
                    )?;
                } else if line.new_lineno().is_none() {
                    write!(
                        w,
                        "<a href=\"#h{}-{}-{}\" id=\"h{}-{}-{}\" class=\"d\">-",
                        i, j, k, i, j, k
                    )?;
                } else {
                    write!(w, " ")?;
                }
                write!(w, "{}", String::from_utf8(line.content().to_vec())?)?;
                if line.old_lineno().is_none() || line.new_lineno().is_none() {
                    write!(w, "</a>")?;
                }
            }
        }
    }
    Ok(())
}
