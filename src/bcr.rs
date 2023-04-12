//TODO: Only one page available at scrapping, needs update when multiple pages are found

use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url = "https://erstegroup-careers.com/bcr/search/?createNewAlert=false&q=";
    let response = reqwest::get(url).await?;
    let html = response.text().await?;

    let document = Html::parse_document(&html);

    let job_selector = Selector::parse("tr.data-row").unwrap();
    let title_selector = Selector::parse(".jobTitle-link").unwrap();
    let location_selector = Selector::parse(".jobShifttype").unwrap();

    let mut jobs = Vec::new();

    for job_element in document.select(&job_selector) {
        let title_element = job_element.select(&title_selector).next().unwrap();
        let title = title_element.text().collect::<String>();
        let link = title_element.value().attr("href").unwrap();

        let location_element = job_element.select(&location_selector).next().unwrap();
        let location = location_element.text().collect::<String>();
        let split: Vec<&str> = link.rsplit('/').collect();
        let content = split[1];
        let job = json!({
            "id": content,
            "job_title": title,
            "job_link": format!("https://erstegroup-careers.com{}", link),
            "company": "BCR",
            "country": "Romania",
            "location": location
        });

        jobs.push(job);
    }

    let json_str = serde_json::to_string_pretty(&jobs)?;
    let mut file = File::create("bcr.json")?;
    file.write_all(json_str.as_bytes())?;
    let end = SystemTime::now();
    let duration = end.duration_since(start).expect("Time went backwards");

    let elapsed_seconds = duration.as_secs_f64();
    let formatted_seconds = if elapsed_seconds < 1.0 {
        format!("{:.2}", elapsed_seconds)
    } else {
        format!("{:.2}", elapsed_seconds)
    };

    println!(
        "Parsed BCR - Jobs found: {} - Took: {}s",
        document.select(&job_selector).count(),
        formatted_seconds
    );

    Ok(())
}
