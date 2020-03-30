
use reqwest::blocking::get;
use std::{fs::read_to_string, path::PathBuf};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use toml::{to_string, from_str};
use std::fs::read_dir;
type Res<T> = Result<T, Box<dyn std::error::Error>>;

mod models;
use models::{StateReport};
fn main() -> Res<()> {
    let all_states = get_new_list()?;
    let now = get_mn(&all_states).expect("MN not on list");
    
    let base = ensure_base_path()?;
    let yesterday = get_yesterday(&base.join(now.yesterday_folder()))?;
    let rep = Report::new(now.modified_local(),
        now.total.unwrap_or(0),
        now.positive.unwrap_or(0),
        yesterday.map(|r| r.info.positive));
    let check_toml = to_string(&now)?;
    let report_toml = to_string(&rep)?;
    let today_dir = base.join(now.folder());
    let now_path = today_dir.join(now.file_name());
    std::fs::create_dir_all(&today_dir)?;
    let report_dir = today_dir.join("report.toml");
    std::fs::write(&now_path, &check_toml).expect("failed to write check file");
    std::fs::write(&report_dir, &report_toml).expect("failed to write daily report");
    if let Ok(arg) = std::env::var("COVID_SEND_EMAIL") {
        if arg != "" && arg != "0" {
            mail(&report_toml, &check_toml)?;
        }
    }
    update_long_report(&base, &today_dir)?;
    Ok(())
}

fn update_long_report(base: &PathBuf, today_path: &PathBuf) -> Res<()> {
    let mut ret = Vec::new();
    
    for ent in read_dir(base)? {
        let ent = ent?;
        if ent.metadata()?.is_dir() {
            let report_path = base.join(ent.path()).join("report.toml");
            let s = std::fs::read_to_string(&report_path)?;
            let report: Report = toml::from_str(&s)?;
            ret.push(models::GraphDot {
                when: report.info.as_of,
                count: report.info.positive,
                ratio: report.ratio.map(|r| r.yesterday).unwrap_or(1.0),
            })
        }
    }
    ret.sort_by(|lhs, rhs| lhs.when.cmp(&rhs.when));
    let points: Vec<(f64, f64)> = ret.iter().enumerate().map(|(i, v)| (i as f64, v.count as f64)).collect();
    println!("{:#?}", ret);
    let page = flot::Page::new("");
    let plot = page.plot("covid MN over time");
    plot.lines("points", points);

    let x_axis: Vec<String> = ret.iter().map(|v| v.when.format("%D").to_string()).collect();
    let x: Vec<(f64, &str)> = x_axis.iter().enumerate().map(|(i, v)| (i as f64, v.as_str())).collect();
    plot.xaxis()
        .tick_values_and_labels(x.as_slice());
    page.render("report.html")?;
    let dest = today_path.join("report.html");
    std::fs::copy("report.html", &dest)?;
    std::fs::remove_file("report.html")?;
    if let Ok(publish) = std::env::var("COVID_REPORT_HTML") {
        std::fs::copy(&dest, &publish)?;
    }
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

fn ensure_base_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
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
#[cfg(target_os = "linux")]
fn mail(report: &str, state: &str) -> Res<()> {
    use lettre::{SendableEmail, EmailAddress, Transport, Envelope, SmtpClient};
    use lettre::SmtpTransport;
    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("r@robertmasen.pizza".to_string())?),
            vec![EmailAddress::new("r.f.masen@gmail.com".to_string())?]
        )?,
        Local::now().to_rfc2822(),
        format!("{}\n\n{}", report, state).as_bytes().to_vec()
    );
    
    // Open a local connection on port 25
    let mut mailer = SmtpTransport::new(SmtpClient::new_unencrypted_localhost()?);
    // Send the email
    mailer.send(email)?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn mail(report: &str, state: &str) -> Res<()> {
    eprintln!("not sending email\n{}\n{}", report, state);
    Ok(())
}