use reqwest::Error;
use serde_derive::{Serialize, Deserialize};
use serde_json::{Value, to_string_pretty};
use std::fs::File;
use std::io::prelude::*;
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
struct RequestBody {
    applied_facets: AppliedFacets,
    limit: i32,
    offset: i32,
    searchText: String,
}

#[derive(Serialize)]
struct AppliedFacets {
    // Add fields as needed
}

async fn fetch_jobs(url: &str, company_name: &str, country_name: &str, post_data: RequestBody) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new();
    let response = client.post(url)
        .json(&post_data)
        .send()
        .await?;
        
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
            company: company_name.to_string(),
            country: country_name.to_string(),
            city: unidecode(&city),
        });
    }

    Ok(result)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let url = "";
    let company_name = "";
    let country_name = "Romania";
    let output_file = ".json";
    
    let post_data = RequestBody {
        applied_facets: AppliedFacets {
            // Fill in the fields as needed
        },
        limit: 10,
        offset: 0,
        searchText: String::from(""),
    };

    match fetch_jobs(url, company_name, country_name, post_data).await {
        Ok(jobs) => {
            println!("Parsed {} - Jobs found: {:?}", company_name, jobs.len());

            let mut file = File::create(output_file)?;
            file.write_all(to_string_pretty(&jobs)?.as_bytes())?;
        }
        Err(e) => {
            println!("Error fetching jobs for {}: {:?}", company_name, e);
        }
    }

    Ok(())
}
