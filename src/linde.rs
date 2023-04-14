use anyhow::Result;
use reqwest::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;
use unidecode::unidecode;
use regex::Regex;

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

async fn fetch_jobs( url: &str, company_name: &str, country_name: &str, post_data: RequestBody,token: &str) -> Result<Vec<Job>, Error> {
    let client = reqwest::Client::new(); 
    let response = client.post(url).header("authorization",format!("Bearer {}",token)).json(&post_data).send().await?;

    let data: Value = response.json().await?;
    let jobs = data["data"]["requisitions"].as_array().unwrap();
    let mut result = Vec::new();

    for job in jobs {
        let shortcode = job["requisitionId"].as_i64().unwrap();
        let title = job["displayJobTitle"].as_str().unwrap();
        let url = format!("https://linde.csod.com/ux/ats/careersite/20/home/requisition/{}",shortcode);

        for city in job["locations"].as_array().unwrap() {
            //panic!("{:?}",city);
            if city["country"].as_str().unwrap() == "RO" {
 
                let mut location = match city["city"].as_str() {
                    Some(city) => city,
                    None => continue,
            
                };
                if location == "Bucharest" {
                    location = "Bucuresti";
                }
                result.push(Job {
                    id: shortcode.to_string(),
                    job_title: title.to_string(),
                    job_link: url.to_string(),
                    company: company_name.to_string(),
                    country: country_name.to_string(),
                    city: unidecode(&location),
                });
            } 
        }  
    }

    Ok(result)

}

async fn job_count(token: &str) -> Result<u64, Error> {
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
    let response = client.post(url).header("authorization",format!("Bearer {}",token)).json(&data).send().await?;
    let json_data: Value = response.json().await?;
    let job_count = json_data["data"]["totalCount"].as_u64().unwrap();

    Ok(job_count)
}

async fn get_token() -> Result<String, Error> {
    let url = "https://linde.csod.com/ux/ats/careersite/20/home?c=linde&country=ro";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?; 
    let html = response.text().await?;
    let re = Regex::new(r#""token":"([^"]+)"#).unwrap();
    let token = match re.captures(&html) {
        Some(captures) => captures.get(1).map_or("".to_string(), |m| m.as_str().to_string()), 
        None => "".to_string(),
    };

    Ok(token)
}


pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let url = "https://eu-fra.api.csod.com/rec-job-search/external/jobs";
    let company_name = "Linde";
    let country_name = "Romania";
    let output_file = "linde.json";
    let token = get_token().await?;
    //    panic!("{}", token);
    let job_count = job_count(&token).await.unwrap();
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
        futures.push(fetch_jobs(url, company_name, country_name, post_data,&token));
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
