use crate::config::Config;
use crate::data::{get_commitinfo, CommitInfo};
use crate::error::AppError;
use crate::handlers::{footer, header};
use crate::util::{print_time, xmlencode, xmlencodeline};
use anyhow::{anyhow, Result};
use axum::{extract::Path, response::Html};
use git2::{Delta, DiffFlags, Patch, Repository};
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

fn print_commit<W: Write>(w: &mut W, ci: &CommitInfo) -> Result<()> {
    write!(w, "<b>commit</b> ")?;
    write!(w, "<a href=\"../commit/{}\">{}</a>\n", ci.oid, ci.oid)?;
    if let Some(poid) = &ci.parentoid {
        write!(w, "<b>parent</b> ")?;
        write!(w, "<a href=\"../commit/{}\">{}</a>\n", poid, poid)?;
    }
    write!(w, "<b>Author:</b> ")?;
    write!(w, "{}", xmlencode(ci.author.name().unwrap_or("")))?;
    let email = xmlencode(ci.author.email().unwrap_or(""));
    write!(w, " <<a href=\"mailto:{}]\">{}</a>>\n", email, email)?;
    write!(w, "<b>Date:</b>   ")?;
    print_time(w, ci.author.when())?;
    write!(w, "\n")?;
    if let Some(msg) = &ci.msg {
        write!(w, "\n{}\n", xmlencode(msg))?;
    }
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
            xmlencode(
                delta
                    .old_file()
                    .path()
                    .unwrap_or(std::path::Path::new(""))
                    .display()
                    .to_string()
                    .as_ref()
            )
        )?;
        if delta.old_file().path() != delta.new_file().path() {
            write!(
                w,
                " -> {}",
                xmlencode(
                    delta
                        .old_file()
                        .path()
                        .unwrap_or(std::path::Path::new(""))
                        .display()
                        .to_string()
                        .as_ref()
                )
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

        let old_file = xmlencode(
            delta
                .old_file()
                .path()
                .unwrap_or(std::path::Path::new(""))
                .display()
                .to_string()
                .as_ref(),
        );
        let new_file = xmlencode(
            delta
                .new_file()
                .path()
                .unwrap_or(std::path::Path::new(""))
                .display()
                .to_string()
                .as_ref(),
        );
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
            write!(
                w,
                "{}",
                xmlencode(String::from_utf8(hunk.header().to_vec())?.as_ref())
            )?;
            write!(w, "</a>")?;

            let mut k = 0;
            loop {
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
                write!(
                    w,
                    "{}",
                    xmlencodeline(
                        String::from_utf8(line.content().to_vec())?.as_ref()
                    )
                )?;
                write!(w, "\n")?;
                if line.old_lineno().is_none() || line.new_lineno().is_none() {
                    write!(w, "</a>")?;
                }
                k += 1;
            }
        }
    }
    Ok(())
}
