use acts_channel::{model::PageData, ActionResult};
use chrono::prelude::*;
use std::error::Error;

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

pub fn local_time(millis: i64) -> String {
    if millis == 0 {
        return "(nil)".to_string();
    }
    match Local.timestamp_millis_opt(millis) {
        chrono::LocalResult::Single(dt) => format!("{}", dt.format("%Y-%m-%d %H:%M:%S")),
        _ => "".to_string(),
    }
}

pub fn size(bits: u32) -> String {
    let mut ret = String::new();
    if bits < 1024 {
        ret.push_str(&format!("{}b", bits));
    } else {
        let kb = bits / 1024;
        if kb < 1024 {
            ret.push_str(&format!("{}kb", kb));
        } else {
            let m = kb / 1024;
            if m < 1024 {
                ret.push_str(&format!("{}m", m));
            }
        }
    }

    ret
}

pub fn print_pager<T>(out: &mut String, data: &PageData<T>) {
    out.push_str(&format!(
        "total {}, page {} of {} ",
        data.count, data.page_num, data.page_count
    ));
}

pub fn print_cost<T>(out: &mut String, resp: &ActionResult<PageData<T>>) {
    let cost = resp.end_time - resp.start_time;
    out.push_str(&format!("(elapsed {cost}ms)"));
}

pub fn parse_sort(s: &str) -> Result<(String, bool), Box<dyn Error + Send + Sync + 'static>> {
    let mut is_rev = false;
    let mut key = s;
    if let Some(pos) = s.find(',') {
        is_rev = &s[pos + 1..] == "desc";
        key = &s[..pos];
    }
    Ok((key.to_string(), is_rev))
}

pub fn parse_key_value(
    s: &str,
) -> Result<(String, String), Box<dyn Error + Send + Sync + 'static>> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

pub fn parse_key_json<T>(
    s: &str,
) -> Result<(T, serde_json::Value), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

    let mut v = s[pos + 1..].to_string();
    let re_not_str =
        regex::Regex::new(r#"([+-]?\d+(\.\d+)?([Ee][+-]?\d+)?)|(\{.*\})|(\[.*\])|null"#).unwrap();
    if !re_not_str.is_match(&v) {
        v = format!(r#""{v}""#);
    }
    Ok((s[..pos].parse()?, serde_json::from_str(&v)?))
}
