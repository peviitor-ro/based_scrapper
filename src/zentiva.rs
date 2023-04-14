use reqwest::Error;
use serde_derive::{Serialize, Deserialize};
use serde_json::{Value, to_string_pretty};
use std::fs::File;
use std::io::prelude::*;
use unidecode::unidecode;
use std::time::SystemTime;

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
#[derive(Serialize)]
struct RequestBody {
    appliedFacets: appliedFacets,
    limit: i32,
    offset: u64,
    searchText: String,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[derive(Serialize)]
struct appliedFacets {
    // Add fields as needed
    locations: Vec<String>,
}


async fn fetch_jobs(url: &str, company_name: &str, country_name: &str, post_data: RequestBody) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new();
    let response = client.post(url)
        .json(&post_data)
        .send()
    .await?;

    let data: Value = response.json().await?;
    let jobs = data["jobPostings"].as_array().unwrap();
    let mut result = Vec::new();
    for job in jobs {
        let id = job["bulletFields"][0][0].as_str().unwrap().to_string();
        let title = job["title"].as_str().unwrap();
        let url = format!("https://zentiva.wd3.myworkdayjobs.com/en-US/Zentiva{}?locations=ca7924da36fa0149be9376945a35dd27", job["externalPath"].as_str().unwrap());
let mut city = job["bulletfields"][1]
        .as_str()
        .and_then(|s| s.split(" / ").nth(1))
        .map(|s| s.to_string())
        .unwrap_or("Romania".to_string());
        if city == "Bucharest" {
            city = "Bucuresti".to_string();
        }
                
        result.push(Job {
            id, 
            job_title: title.to_string(),
            job_link: url.to_string(),
            company: company_name.to_string(),
            country: country_name.to_string(),
            city: unidecode(&city),
        });
    }

    Ok(result)
}
async fn job_count (url: &str) -> Result<u64,Error> {
    let client = reqwest::Client::new();
    let post_data = RequestBody {
        appliedFacets: appliedFacets {
            locations: vec!["ca7924da36fa0149be9376945a35dd27".to_owned()]
        },
        limit: 20,
        offset: 0,
        searchText: "".to_owned(),
    };

    let response = client.post(url)
        .json(&post_data)
        .send()
    .await?;

    let data: Value = response.json().await?;
    let job_count = data["total"].as_u64().unwrap();
    Ok(job_count) 

}
pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url = "https://zentiva.wd3.myworkdayjobs.com/wday/cxs/zentiva/Zentiva/jobs".to_string();
    let company_name = "Zentiva";
    let country_name = "Romania";
    let output_file = "zentiva.json";
    let mut offset = 0;
    let job_count = job_count(&url).await.unwrap();
    let mut job_results = Vec::new();
    let mut futures = Vec::new(); // Create a vector to store the futures
    while job_count < offset {
        let post_data = RequestBody {
            appliedFacets: appliedFacets {
                locations: vec!["ca7924da36fa0149be9376945a35dd27".to_owned()]
            },
            limit: 20,
            offset,
            searchText: "".to_owned(),
        };

        futures.push(fetch_jobs(&url, company_name, country_name, post_data));
        offset +=20;
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

