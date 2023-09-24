use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use git2::Time;
use std::fmt::Write;

pub fn print_time<W: Write>(w: &mut W, intime: Time) -> Result<()> {
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

pub fn print_time_short<W: Write>(w: &mut W, intime: Time) -> Result<()> {
    let dt = DateTime::from_timestamp(intime.seconds(), 0)
        .ok_or(anyhow!("Error parsing timestamp seconds: {:#?}", intime))?;
    let fmt_dt = dt.format("%Y-%m-%d %H:%M");
    write!(w, "{}", fmt_dt)?;
    return Ok(());
}

/// Escape characters below as HTML 2.0 / XML 1.0
pub fn xmlencode(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt"),
            '>' => result.push_str("&gt;"),
            '\'' => result.push_str("&#39;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape characters below as HTML 2.0 / XML 1.0, ignoring '\n' and '\r'
pub fn xmlencodeline(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt"),
            '>' => result.push_str("&gt;"),
            '\'' => result.push_str("&#39;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\r' | '\n' => (),
            _ => result.push(c),
        }
    }
    result
}
