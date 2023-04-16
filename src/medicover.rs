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

async fn job_count() -> Result<u64, Error> {
    let url = "https://mingle.ro/api/boards/mingle/jobs?q=companyUid~eq~%22medicover%22&page=0&pageSize=1000&sort=modifiedDate~DESC";
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let jobs = data["data"]["results"].as_array().unwrap();
    Ok(jobs.len() as u64)
}

async fn fetch_jobs(url: &str, company_name: &str, country_name: &str) -> Result<Vec<Job>, Error> {
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let jobs = data["data"]["results"].as_array().unwrap();
    let mut result = Vec::new();

    for job in jobs {
        let shortcode = job["id"].as_i64().unwrap();
        let title = job["jobTitle"].as_str().unwrap();
        let url_part = job["publicUid"].as_str().unwrap();
        let url = format!("https://medicover.mingle.ro/en/apply/{}", url_part);
        let mut city = job["locations"][0]["name"].as_str().unwrap_or("Romania");
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

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url="https://mingle.ro/api/boards/mingle/jobs?q=companyUid~eq~%22medicover%22&page=0&pageSize=1000&sort=modifiedDate~DESC";
    let company_name = "Medicover";
    let country_name = "Romania";
    let output_file = "medicover.json";

    match fetch_jobs(url, company_name, country_name).await {
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
                "Parsed {} - Jobs found: {:?} - Took {}s",
                company_name,
                jobs.len(),
                formatted_seconds
            );

            let mut file = File::create(output_file)?;
            file.write_all(to_string_pretty(&jobs)?.as_bytes())?;
        }
        Err(e) => {
            println!("Error fetching jobs for {}: {:?}", company_name, e);
        }
    }

     
    Ok(())
}
