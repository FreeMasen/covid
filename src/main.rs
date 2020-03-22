
use reqwest::blocking::get;
use std::{fs::read_to_string, path::PathBuf};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use toml::{to_string, from_str};
type Res<T> = Result<T, Box<dyn std::error::Error>>;
fn main() -> Res<()> {
    let html = get("https://www.health.state.mn.us/diseases/coronavirus/situation.html")?
        .text()?;
    let (tested, _new_start) = find_tested(&html).expect("failed to find tested");
    let (positive, _new_start) = find_positive(&html).expect(&format!("failed to find positive\n{}", html));
    let (as_of, _) = find_as_of(&html).expect("failed to find as of date");
    let file_name = format!("{}.toml", as_of.format("%Y-%m-%dT%H:%M:%S"));
    let base = ensure_path()?;
    let yesterday = get_yesterday(&file_name, &base)?;
    let rep = Report::new(as_of, tested, positive, yesterday.map(|r| r.info.positive));
    let t = to_string(&rep)?;
    

    std::fs::write(base.join(&file_name), t).expect("failed to write file");
    Ok(())
}

fn get_yesterday(file_name: &str, report_dir: &PathBuf) -> Res<Option<Report>> {
    let rd = std::fs::read_dir(report_dir)?;
    let mut files: Vec<String> = rd.filter_map(|d| {
        if let Ok(ent) = d {
            if let Some(n) = ent.file_name().to_str() {
                Some(n.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }).collect();
    files.sort();
    println!("files in output: {:?}", files);
    if let Some(last) = files.last() {
        if last != file_name {
            let s = read_to_string(report_dir.join(&last))?;
            let report = from_str(&s)?;
            Ok(Some(report))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn ensure_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var("COVID_OUTPUT_DIR")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|_| {
        let base = std::env::current_dir().unwrap_or(PathBuf::new());
        base.join("output")
    });
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn find_as_of<'a>(html: &'a str) -> Option<(DateTime<Local>, usize)> {
    let updated = "Updated";
    let start = html.rfind(updated)?;
    let target = html[start+updated.len()..].trim();
    let end = target.find("</p>")?;
    let dt_str = target[..end].trim_end();
    println!("parsing {:?}", dt_str);
    let dt = parse_date(dt_str)?;
    Some((dt, start + end))
}

fn parse_date(dt: &str) -> Option<DateTime<Local>> {
    println!("parsing date: {}", dt);
    let comma_idx = dt.find(",").expect("comma");
    let target = dt[comma_idx..].trim();
    let (day, len) = parse_number(target).expect("day");
    let target = target[len..].trim_start_matches("-");
    println!("parsing month: {}", target);
    let  month = parse_month(target).expect("month");
    let target = target[3..].trim_start_matches("-");
    let (year, len) = parse_number(target).expect("year");
    let target = target[len..].trim();
    let (hour, len) = parse_number(target).expect("hour");
    let target = target[len..].trim_start_matches(":");
    let (min, len) = parse_number(target).expect("minute");
    let target = target[len..].trim_start_matches(":");
    let (sec, _) = parse_number(target).expect("second");
    Some(Local.ymd(year as _, month as _, day as _).and_hms(hour as _, min as _, sec as _))

}

fn find_tested(html: &str) -> Option<(u32, usize)> {
    let tested_prefix = "Approximate number of patients tested at";
    let start = html.find(tested_prefix)?;
    let target = &html[start+tested_prefix.len()..];
    let (num, ct) = parse_number(target)?;
    Some((num, start + ct))
}

fn find_positive(html: &str) -> Option<(u32, usize)> {
    let positve_prefix = "Positive:";
    let start = html.find(positve_prefix)?;
    let target = &html[start+positve_prefix.len()..];
    let (num, ct) = parse_number(target)?;
    Some((num, start + ct))
}

fn parse_number(s: &str) -> Option<(u32, usize)> {
    let mut ct = 0;
    let mut ch = s.chars();
    let mut num = loop {
        let c = ch.next()?;
        ct += 1;
        if c.is_digit(10) {
            break c.to_digit(10).unwrap()
        }
    };
    while let Some(c) = ch.next() {
        if c.is_digit(10) {
            ct += 1;
            num = (num * 10) + c.to_digit(10).unwrap();
        } else {
            break;
        }
    }
    Some((num, ct))
}

fn parse_month(s: &str) -> Option<u32> {
    Some(match &s[..3] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    })
}
#[derive(Serialize, Deserialize)]
struct Report {
    info: Info,
    ratio: Option<Ratio>,
}

impl Report {
    pub fn new(as_of: DateTime<Local>, tested: u32, positive: u32, prev_positive: Option<u32>) -> Self {
        
        let ratio = if let Some(prev) = prev_positive {
            let raw = positive as f32 / prev as f32;
            Some(Ratio {
                yesterday: raw,
                prev_positive: prev
            })
        } else {
            None
        };
        Self {
            info: Info {
                as_of,
                tested,
                positive
            },
            ratio
        }
    }
}
#[derive(Serialize, Deserialize)]
struct Info {
    as_of: DateTime<Local>,
    tested: u32,
    positive: u32,
}
#[derive(Serialize, Deserialize)]
struct Ratio {
    yesterday: f32,
    prev_positive: u32,
}
