use futures::future::join_all;
use reqwest::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;

#[derive(Deserialize)]
struct JobList {
    positions: Vec<Job>,
    count: u64,
}

#[derive(Deserialize)]
struct Job {
    id: u64,
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
struct Jobs {
    id: u64,
    job_title: String,
    job_link: String,
    company: String,
    country: String,
    city: String,
}
async fn fetch_job_list(url: String) -> Result<JobList, Error> {
    let job_list: JobList = reqwest::get(&url).await?.json().await?;
    Ok(job_list)
}
pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();

    let base_url =
        "https://jobs.vodafone.com/api/apply/v2/jobs?domain=vodafone.com&location=Romania";
    let num = 10;
    let url = format!("{}&start=0&num={}", base_url, num);
    let job_list: JobList = reqwest::get(&url).await?.json().await?;
    let count = job_list.count;
    let iterations = (count + num - 1) / num; // This will round up the division
    let mut tasks = Vec::new();
    for i in 0..iterations {
        let start = i * num;
        let url = format!("{}&start={}&num={}", base_url, start, num);
        tasks.push(tokio::spawn(fetch_job_list(url)));
    }
    let results = join_all(tasks).await;
    let mut jobs = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(join_handle_result) => match join_handle_result {
                Ok(JobList { positions, .. }) => {
                    for position in positions {
                        let mut job_link = String::from("https://jobs.vodafone.com/careers?pid=");
                        job_link.push_str(&position.id.to_string());
                        let mut job_city = position.location.split(",");

                        let job = Jobs {
                            id: position.id,
                            job_title: position.name,
                            job_link: job_link,
                            company: "Vodafone".to_string(),
                            country: "Romania".to_string(),
                            city: job_city.next().unwrap_or("").to_string(),
                        };
                        jobs.push(job);
                    }
                }
                Err(e) => {
                    println!("Error fetching jobs for batch {}: {:?}", i + 1, e);
                }
            },
            Err(e) => {
                println!("Error joining task for batch {}: {:?}", i + 1, e);
            }
        }
    }
    let json_data = to_string_pretty(&jobs)?;
    let mut file = File::create("vodafone.json")?;
    file.write_all(json_data.as_bytes())?;
    let end = SystemTime::now();
    let duration = end.duration_since(start).expect("Time went backwards");
    let elapsed_seconds = duration.as_secs_f64();
    let formatted_seconds = if elapsed_seconds < 1.0 {
        format!("{:.2}", elapsed_seconds)
    } else {
        format!("{:.2}", elapsed_seconds)
    };

    println!(
        "Parsed Vodafone - Jobs found: {} - Took: {}s",
        count, formatted_seconds
    );
    Ok(())
}
