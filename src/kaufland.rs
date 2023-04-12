//TODO: De ce backendu nu returneaza toate jobs pe coords ?

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

#[derive(Serialize, Deserialize)]
struct Location {
    latitude: String,
    longitude: String,
    total: i32,
}

async fn fetch_jobs(
    company_name: &str,
    country_name: &str,
    latitude: String,
    longitude: String,
) -> Result<Vec<Job>, Error> {
    let base_url = "https://rmk-map-12.jobs2web.com/services/jobmap/jobs?siteid=Gi8p27qFoBvZm%2FcnHxoZEQ%3D%3D&mapType=GOOGLE_MAP&jobTitle=&coordinates=45.7684%2C24.1805&locale=ro_RO&brand=Romania&limittobrand=false".to_string();
    let coordinates = format!("{},{}", latitude, longitude);
    let url = base_url.replace(
        "coordinates=43.7522%2C24.8637",
        &format!("coordinates={}", coordinates),
    );
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let jobs = data.as_array().unwrap();
    //panic!("{:#?}", jobs);
    let mut result = Vec::new();
    for job in jobs {
        let shortcode = job["siteid"].to_string();
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

async fn get_coords() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://rmk-map-12.jobs2web.com/services/jobmap/jobs/facets?siteid=Gi8p27qFoBvZm%2FcnHxoZEQ%3D%3D&mapType=GOOGLE_MAP&jobTitle=&locale=ro_RO&brand=Romania&limittobrand=true";
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;

    let json_string = serde_json::to_string(&data)?;
    Ok(json_string)
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let company_name = "Kaufland";
    let country_name = "Romania";
    let output_file = "kaufland.json";

    let coords = get_coords().await.unwrap();

    let mut job_results = Vec::new();
    let mut futures = Vec::new();
    let deserialized_array: Vec<Location> = serde_json::from_str(&coords).unwrap();
    for location in deserialized_array.into_iter() {
        futures.push(fetch_jobs(
            company_name,
            country_name,
            location.latitude,
            location.longitude,
        ));
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
