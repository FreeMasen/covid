
use reqwest::blocking::get;
use std::{fs::read_to_string, path::PathBuf};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use toml::{to_string, from_str};
type Res<T> = Result<T, Box<dyn std::error::Error>>;

mod models;
use models::{StateReport};
fn main() -> Res<()> {
    let all_states = get_new_list()?;
    let now = get_mn(&all_states).expect("MN not on list");
    
    let base = ensure_path()?;

    let yesterday = get_yesterday(&base.join(now.yesterday_folder()))?;
    let rep = Report::new(now.modified_local(),
        now.total.unwrap_or(0),
        now.positive.unwrap_or(0),
        yesterday.map(|r| r.info.positive));
    let check_toml = to_string(&now)?;
    let report_toml = to_string(&rep)?;
    std::fs::write(&now.extend_path(&base), check_toml).expect("failed to check write file");
    std::fs::write(&base.join(now.folder()).join("report.toml"), report_toml).expect("failed to write daily report");
    Ok(())
}

fn get_mn(all_states: &[StateReport]) -> Option<StateReport> {
    for state in all_states {
        if state.state == "MN" {
            return Some(state.clone())
        }
    }
    None
}

fn get_new_list() -> Res<Vec<StateReport>> {
    let list = get("https://covidtracking.com/api/states")?
        .json()?;
    Ok(list)
}

fn get_yesterday(dir: &PathBuf) -> Res<Option<Report>> {
    let file = dir.join("report.toml");
    if !file.exists() {
        eprintln!("{:?} does't exist for yesterday", file);
        return Ok(None)
    }
    let s = read_to_string(&file)?;
    let report = from_str(&s)?;
    Ok(Some(report))
       
}

fn yesterdays_dir(base: &PathBuf) -> PathBuf {
    let today = Local::today();
    let yesterday = today - chrono::Duration::days(1);
    PathBuf::from(&format!("{}.{:02}.{:02}", yesterday.year(), yesterday.month(), yesterday.day()))
}

fn ensure_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var("COVID_OUTPUT_DIR")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|_| {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
        base.join("output")
    });
    std::fs::create_dir_all(&path)?;
    Ok(path)
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
