//TODO: API key is hardcoded, need to find a way to get it from the website
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

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
struct RequestBody {
    careerSiteId: u32,
    careerSitePageId: u32,
    pageNumber: u64,
    pageSize: u32,
    cultureId: u32,
    searchText: String,
    cultureName: String,
    states: Vec<String>,
    countryCodes: Vec<String>,
    cities: Vec<String>,
    placeID: String,
    radius: Option<u32>,
    postingsWithinDays: Option<u32>,
    customFieldCheckboxKeys: Vec<String>,
    customFieldDropdowns: Vec<String>,
    customFieldRadios: Vec<String>,
}

async fn fetch_jobs(url: &str, company_name: &str, country_name: &str, post_data: RequestBody) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new();
    let response = client.post(url)
        .header("Authorization", "Bearer eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCIsImNsaWQiOiIxZjhpdW4xY29neHAifQ.eyJjY2lkIjoiZm5pa21ucHd3c3F4aGF1Iiwic3ViIjotMTE5LCJhdWQiOiJoejR0NWt1NXQzaXptY2piMTNqNXZhNGIiLCJjb3JwIjoibGluZGUiLCJjdWlkIjoxLCJ0emlkIjoxLCJuYmQiOiIyMDIzMDQxMjEwMjY0OTQ4MSIsImV4cCI6IjIwMjMwNDEyMTEyNzQ5NDgxIiwiaWF0IjoiMjAyMzA0MTIxMDI2NDk0ODEifQ.rvf2ZnT2r8rWkAPTzG_SKKdlnJGnB5QBKraJK7oN-yxrzKGGfDLFXGKVtC1QzUn5HUP5hGkGkC_wlWVKL6pAeQ")
        .header("Content-Type", "application/json")
        .json(&post_data)
        .send()
    .await?;

    let data: Value = response.json().await?;
    let jobs = data["data"]["requisitions"].as_array().unwrap();
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


async fn job_count() -> Result<u64,Error> {
    let url = "https://eu-fra.api.csod.com/rec-job-search/external/jobs";
    let data = RequestBody {
        careerSiteId: 20,
        careerSitePageId: 20,
        pageNumber: 1,
        pageSize: 25,
        cultureId: 1,
        searchText: String::from(""),
        cultureName: String::from("en-US"),
        states: vec![],
        countryCodes: vec![String::from("ro")],
        cities: vec![],
        placeID: String::from(""),
        radius: None,
        postingsWithinDays: None,
        customFieldCheckboxKeys: vec![],
        customFieldDropdowns: vec![],
        customFieldRadios: vec![],
    };
   
    let client = reqwest::Client::new();
    let response = client.post(url)
        .json(&data)
        .header("Authorization", "Bearer eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCIsImNsaWQiOiIxZjhpdW4xY29neHAifQ.eyJjY2lkIjoiZm5pa21ucHd3c3F4aGF1Iiwic3ViIjotMTE5LCJhdWQiOiJoejR0NWt1NXQzaXptY2piMTNqNXZhNGIiLCJjb3JwIjoibGluZGUiLCJjdWlkIjoxLCJ0emlkIjoxLCJuYmQiOiIyMDIzMDQxMjEwMjY0OTQ4MSIsImV4cCI6IjIwMjMwNDEyMTEyNzQ5NDgxIiwiaWF0IjoiMjAyMzA0MTIxMDI2NDk0ODEifQ.rvf2ZnT2r8rWkAPTzG_SKKdlnJGnB5QBKraJK7oN-yxrzKGGfDLFXGKVtC1QzUn5HUP5hGkGkC_wlWVKL6pAeQ")
        .header("Content-Type", "application/json")
        .send()
    .await?;
    
    let json_data: Value = response.json().await?;
 
    let job_count = json_data["data"]["totalCount"].as_u64().unwrap();


    Ok(job_count)


}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://eu-fra.api.csod.com/rec-job-search/external/jobs";
    let company_name = "Linde";
    let country_name = "Romania";
    let output_file = "linde.json";

    let job_count = job_count().await.unwrap();
    let mut job_results = Vec::new();
    let total_pages = (job_count + 19) / 20; // Calculate the total number of pages
    let mut futures = Vec::new(); // Create a vector to store the futures

    for current_page in 1..=total_pages {
        let post_data = RequestBody {
            careerSiteId: 20,
            careerSitePageId: 20,
            pageNumber: current_page,
            pageSize: 25,
            cultureId: 1,
            searchText: String::from(""),
            cultureName: String::from("en-US"),
            states: vec![],
            countryCodes: vec![String::from("ro")],
            cities: vec![],
            placeID: String::from(""),
            radius: None,
            postingsWithinDays: None,
            customFieldCheckboxKeys: vec![],
            customFieldDropdowns: vec![],
            customFieldRadios: vec![],
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

    println!("Parsed {} - Jobs found: {:?}", company_name, job_results.len());

    let mut file = File::create(output_file)?;
    file.write_all(to_string_pretty(&job_results)?.as_bytes())?;

    Ok(())
}
