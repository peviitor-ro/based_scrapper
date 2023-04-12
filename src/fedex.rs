use futures::{stream, StreamExt};
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

async fn fetch_jobs(
    url: String,
    company_name: &str,
    country_name: &str,
) -> Result<Vec<Job>, Error> {
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let jobs = data["jobs"].as_array().unwrap();
    let mut result = Vec::new();

    for job in jobs {
        let shortcode = job["data"]["slug"].as_str().unwrap();
        let title = job["data"]["title"].as_str().unwrap();
        let url = job["data"]["apply_url"].as_str().unwrap();
        let mut city = job["data"]["city"].as_str().unwrap();
        if city == "Bucharest" {
            city = "Bucuresti";
        }
        result.push(Job {
            id: shortcode.to_string(),
            job_title: title.to_string(),
            job_link: url.to_string(),
            company: company_name.to_string(),
            country: country_name.to_string(),
            city: unidecode(&city),
        });
    }

    Ok(result)
}

async fn job_count(url: String) -> Result<u64, Error> {
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let count = data["totalCount"].as_u64().unwrap();
    Ok(count)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url="https://careers.fedex.com/api/jobs?location=Romania&woe=12&stretchUnit=MILES&stretch=10&page=1&limit=1&brand=FedEx%20Express%20EU".to_string();
    let company_name = "Fedex";
    let country_name = "Romania";
    let output_file = "fedex.json";
    let jobs_count = job_count(url).await.unwrap();
    let mut current_page = 1;
    let mut job_results = Vec::new();
    let mut fetch_jobs_futures = stream::FuturesUnordered::new();
    while (current_page - 1) * 100 < jobs_count {
        let search_url = format!("https://careers.fedex.com/api/jobs?location=Romania&woe=12&stretchUnit=MILES&stretch=10&page={}&limit=100&brand=FedEx%20Express%20EU", current_page);
        fetch_jobs_futures.push(fetch_jobs(search_url, company_name, country_name));
        current_page += 1;
    }
    while let Some(result) = fetch_jobs_futures.next().await {
        match result {
            Ok(mut page_job_results) => {
                job_results.append(&mut page_job_results);
            }
            Err(e) => {
                eprintln!("Error fetching jobs: {:?}", e);
            }
        }
    }

    let end = SystemTime::now();
    let duration = end.duration_since(start).expect("Time went backwards");

    let elapsed_seconds = duration.as_secs_f64();
    let formatted_seconds = if elapsed_seconds < 1.0 {
        format!("{:.2}", elapsed_seconds)
    } else {
        format!("{:.2}", elapsed_seconds)
    };

    println!(
        "Parsed {} - Jobs found: {:?} - Took: {}s",
        company_name,
        job_results.len(),
        formatted_seconds
    );
    let mut file = File::create(output_file)?;
    file.write_all(to_string_pretty(&job_results)?.as_bytes())?;
    Ok(())
}
