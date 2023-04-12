use reqwest::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;
use unidecode::unidecode;
#[derive(Serialize, Deserialize)]
struct Job {
    id: String,
    job_title: String,
    job_link: String,
    company: String,
    country: String,
    city: String,
}

async fn fetch_jobs(url: &str) -> Result<Vec<Job>, Error> {
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let jobs = data["jobs"].as_array().unwrap();
    let mut result = Vec::new();

    for job in jobs {
        let shortcode = job["shortcode"].as_str().unwrap();
        let title = job["title"].as_str().unwrap();
        let url = job["url"].as_str().unwrap();
        let mut city = job["city"].as_str().unwrap();
        if city == "Bucharest" {
            city = "Bucuresti";
        }
        result.push(Job {
            id: shortcode.to_string(),
            job_title: title.to_string(),
            job_link: url.to_string(),
            company: "Decathlon".to_string(),
            country: "Romania".to_string(),
            city: unidecode(&city),
        });
    }

    Ok(result)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();

    let url = "https://apply.workable.com/api/v1/widget/accounts/404273";

    match fetch_jobs(url).await {
        Ok(jobs) => {
            let end = SystemTime::now();
            let duration = end.duration_since(start).expect("Time went backwards");

            let elapsed_seconds = duration.as_secs_f64();
            let formatted_seconds = if elapsed_seconds < 1.0 {
                format!("{:.2}", elapsed_seconds)
            } else {
                format!("{:.2}", elapsed_seconds)
            };

            println!(
                "Parsed Decathlon - Jobs found: {:?} - Took: {}s",
                jobs.len(),
                formatted_seconds
            );

            let mut file = File::create("decathlon.json")?;
            file.write_all(to_string_pretty(&jobs)?.as_bytes())?;
        }
        Err(e) => {
            println!("Error fetching jobs: {:?}", e);
        }
    }

    Ok(())
}
