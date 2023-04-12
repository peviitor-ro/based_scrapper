use anyhow::Result;

use reqwest::ClientBuilder;
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;
use titlecase::titlecase;

#[derive(Serialize, Deserialize, Debug)]
struct Job {
    id: String,
    job_title: String,
    job_link: String,
    company: String,
    country: String,
    city: String,
}

async fn fetch_jobs(
    url: String,
    company_name: String,
    country_name: String,
) -> Result<Vec<Job>, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?;
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let script_selector = Selector::parse("script").unwrap();
    let mut json_data = String::new();

    for script_element in document.select(&script_selector) {
        let script_text = script_element.inner_html();
        if script_text.contains("var vmCfg =") {
            let re = regex::Regex::new(r#"var vmCfg = (\{.*\});"#).unwrap();
            if let Some(captures) = re.captures(&script_text) {
                json_data = captures[1].to_string();
                break;
            }
        }
    }
    let vm_cfg: Value = serde_json::from_str(&json_data)?;
    let position_list = vm_cfg["PositionList"].as_array().unwrap();
    let mut jobs = Vec::new();
    for pos in position_list {
        let job_title = pos["PositionName"].to_string().replace("\"", "");
        let city = titlecase(&pos["CityList"].to_string().replace("\"", ""));
        jobs.push(Job {
            id: pos["PositionId"].to_string(),
            job_title: job_title.into(),
            job_link: format!(
                "https://cariere.auchan.ro/Position/Details?id={}",
                pos["PositionId"]
            ),
            company: company_name.clone(),
            country: country_name.clone(),
            city: city.into(),
        });
    }

    Ok(jobs)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();

    let company_name = "Auchan";
    let country_name = "Romania";
    let output_file = "auchan.json";
    let url = "https://cariere.auchan.ro/";

    let jobs = fetch_jobs(
        url.to_string(),
        company_name.to_string(),
        country_name.to_string(),
    )
    .await?;
    let end = SystemTime::now();
    let duration = end.duration_since(start).expect("Time went backwards");

    let elapsed_seconds = duration.as_secs_f64();
    let formatted_seconds = if elapsed_seconds < 1.0 {
        format!("{:.2}", elapsed_seconds)
    } else {
        format!("{:.2}", elapsed_seconds)
    };

    println!(
        "Parsed {} - Jobs found: {} - Took: {}s",
        company_name,
        jobs.len(),
        formatted_seconds
    );

    let mut file = File::create(output_file)?;
    file.write_all(serde_json::to_string_pretty(&jobs)?.as_bytes())?;
    Ok(())
}
