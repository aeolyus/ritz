use crate::config::Config;
use crate::handlers::{header, footer};
use axum::{extract::Path, response::Html};
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::Repository;

pub async fn refs(Path(repo): Path<String>) -> Html<String> {
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
    result.push_str("<h2>Branches</h2>");
    result.push_str(
        "<table>
        <thead>
        <tr>
        <td><b>Name</b></td>
        <td><b>Last commit date</b></td>
        <td><b>Author</b></td>
        </tr>
        </thead>",
    );
    for reference in repo
        .references()
        .unwrap()
        .filter(|r| r.as_ref().unwrap().is_branch())
    {
        let r = reference.unwrap();
        result.push_str("<tr>");
        result.push_str(&format!(
            "<td>{}</td>",
            &r.shorthand().unwrap().to_string()
        ));
        let commit = r.peel_to_commit().unwrap();
        let naive = NaiveDateTime::from_timestamp(commit.time().seconds(), 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let formatted_datetime = datetime.format("%Y-%m-%d %H:%M");
        result.push_str(&format!("<td>{}</td>", formatted_datetime,));
        result.push_str(&format!("<td>{}</td>", &commit.author().to_string()));
        result.push_str("</tr>");
    }
    result.push_str("</table>");

    result.push_str("<h2>Tags</h2>");
    result.push_str(
        "<table>
        <thead>
        <tr>
        <td><b>Name</b></td>
        <td><b>Last commit date</b></td>
        <td><b>Author</b></td>
        </tr>
        </thead>",
    );
    for reference in repo
        .references()
        .unwrap()
        .filter(|r| r.as_ref().unwrap().is_tag())
    {
        let r = reference.unwrap();
        result.push_str("<tr>");
        result.push_str(&format!(
            "<td>{}</td>",
            &r.shorthand().unwrap().to_string()
        ));
        let commit = r.peel_to_commit().unwrap();
        let naive = NaiveDateTime::from_timestamp(commit.time().seconds(), 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let formatted_datetime = datetime.format("%Y-%m-%d %H:%M");
        result.push_str(&format!("<td>{}</td>", formatted_datetime,));
        result.push_str(&format!("<td>{}</td>", &commit.author().to_string()));
        result.push_str("</tr>");
    }
    result.push_str("</table>");

    result.push_str(footer());
    Html(result)
}
