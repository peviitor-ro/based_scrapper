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

#[allow(non_snake_case)]
#[derive(Clone, Serialize)]
struct RequestBody {
    applied_facets: AppliedFacets,
    limit: i64,
    offset: i64,
    searchText: String,
}

#[derive(Clone, Serialize)]
struct AppliedFacets {
    // Add fields as needed
}
async fn fetch_jobs(
    url: &str,
    company_name: &str,
    country_name: &str,
    post_data: RequestBody,
) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new();

    let mut result = Vec::new();
    let mut offset = 0;

    loop {
        let mut post_data = post_data.clone();
        post_data.offset = offset;
        post_data.searchText = "Romania".to_string();

        let response = client.post(url).json(&post_data).send().await?;

        let data: Value = response.json().await?;
        let total_jobs = data["total"].as_i64().unwrap();
        let jobs = data["jobPostings"].as_array().unwrap();
        for job in jobs {
            let shortcode = job["bulletFields"][0].as_str().unwrap();
            let title = job["title"].as_str().unwrap();
            let url = job["externalPath"].as_str().unwrap();
            let mut city = job["locationsText"]
                .as_str()
                .unwrap()
                .split(',')
                .nth(1)
                .unwrap()
                .trim();
            if city.contains("Bucharest") {
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
        if offset + post_data.limit as i64 >= total_jobs {
            break;
        }

        offset += post_data.limit as i64;
    }

    Ok(result)
}

async fn job_count() -> Result<u64, Box<dyn std::error::Error>> {
    let url = "https://sec.wd3.myworkdayjobs.com/wday/cxs/sec/Samsung_Careers/jobs";
    let post_data = RequestBody {
        applied_facets: AppliedFacets {},
        limit: 10,
        offset: 0,
        searchText: String::from(""),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&post_data).send().await?;
    let data: Value = response.json().await?;
    let total_jobs = data["total"].as_u64().unwrap();
    Ok(total_jobs)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url = "https://sec.wd3.myworkdayjobs.com/wday/cxs/sec/Samsung_Careers/jobs";
    let company_name = "Samsung";
    let country_name = "Romania";
    let output_file = "samsung.json";

    let post_data = RequestBody {
        applied_facets: AppliedFacets {},
        limit: 10,
        offset: 0,
        searchText: String::from(""),
    };
    match fetch_jobs(url, company_name, country_name, post_data).await {
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
                "Parsed {} - Jobs found: {:?} - Took: {}s",
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
