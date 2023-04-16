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

#[derive(Serialize)]
#[allow(non_snake_case)]
struct RequestBody {
    locations: Vec<String>,
    workAreas: Vec<String>,
    contractType: Vec<String>,
    fulltext: String,
    order_by: String,
    page: u64,
}

async fn fetch_jobs(
    url: &str,
    company_name: &str,
    country_name: &str,
    post_data: RequestBody,
) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new();
    let response = client.post(url).json(&post_data).send().await?;

    let data: Value = response.json().await?;
    let jobs = data["jobs"].as_array().unwrap();
    let mut result = Vec::new();

    for job in jobs {
        let shortcode = job["ref_number"].as_str().unwrap();
        let title = job["title"].as_str().unwrap();
        let url = job["apply_on_web_url"].as_str().unwrap();
        let mut city = job["city"].as_str().unwrap();
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

async fn job_count() -> Result<u64, Error> {

    let url = "https://career.hm.com/wp-json/hm/v1/sr/jobs/search?_locale=user";
    let data = RequestBody {
        locations: vec![String::from("cou:ro")],
        workAreas: vec![],
        contractType: vec![],
        fulltext: String::from(""),
        order_by: String::from(""),
        page: 1,
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&data).send().await?;

    let data: Value = response.json().await?;
    let job_count = data["total"].as_u64().unwrap();

    Ok(job_count)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url = "https://career.hm.com/wp-json/hm/v1/sr/jobs/search?_locale=user";
    let company_name = "HM";
    let country_name = "Romania";
    let output_file = "hm.json";
    let job_count = job_count().await.unwrap();
    let mut job_results = Vec::new();
    let total_pages = (job_count + 8) / 9; // Calculate the total number of pages
    let mut futures = Vec::new(); // Create a vector to store the futures
    for current_page in 1..=total_pages {
        let post_data = RequestBody {
            locations: vec![String::from("cou:ro")],
            workAreas: vec![],
            contractType: vec![],
            fulltext: String::from(""),
            order_by: String::from(""),
            page: current_page,
        };

        // Store the futures in the vector
        futures.push(fetch_jobs(url, company_name, country_name, post_data));
    }
    let results = futures::future::join_all(futures).await;

    for res in results {
        match res {
            Ok(jobs) => job_results.extend(jobs),
            Err(e) => eprintln!("Error fetching jobs: {}", e),
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
        "Parsed {} - Jobs found: {:?} - Took {}s",
        company_name,
        job_results.len(),
        formatted_seconds
    );

    let mut file = File::create(output_file)?;
    file.write_all(to_string_pretty(&job_results)?.as_bytes())?;
     
    Ok(())
}
