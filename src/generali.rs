//TODO: Trebuie urmarit cand apar mai multe pentru a deduce paginatia
use serde_json::to_string_pretty;
use anyhow::{Result};
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;  
use futures::{stream, StreamExt};
use std::hash::Hasher;
use twox_hash::XxHash;

#[derive(Serialize, Deserialize, Debug)]
struct Job {
    id: u64,
    job_title: String,
    job_link: String,
    company: String,
    country: String,
    city: String,
}
async fn fetch_jobs(url: String, company_name: String, country_name: String) -> Result<Vec<Job>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36")
        .build()?;
    let response = client.get(url).send().await?;
    let html = response.text().await?;

    let document = Html::parse_document(&html);
    let listing_selector = Selector::parse(".h3-list-job-title").unwrap();
    let mut jobs = Vec::new();

    for listing_element in document.select(&listing_selector) {
        let anchor_selector = Selector::parse("a").unwrap();
        let mut anchor_iter = listing_element.select(&anchor_selector);
        let anchor = anchor_iter.next().unwrap();
        let job_title = anchor.text().collect::<String>();
        let job_link = anchor.value().attr("href").unwrap().to_string();
       
        let mut hasher = XxHash::with_seed(0);
        hasher.write(job_link.as_bytes());
        let link_hash = hasher.finish();


        jobs.push(Job {
            id: link_hash,
            job_title,
            job_link,
            company: company_name.clone(),
            country: country_name.clone(),
            city: country_name.clone(),
        });
    }
    Ok(jobs)
}

async fn job_count() -> Result<u64> {
    
    let url = "https://cariere.generali.ro/jobs/search".to_string();
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36")
        .build()?;
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);

    let selector = Selector::parse(".h3-list-job-title").unwrap();
    let element = document.select(&selector).next().unwrap();
 
    // count the number of elements that have the class "h3-list-job-title"
    let jobs_count = element.select(&selector).count();

    Ok(jobs_count as u64)


}


pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let company_name = "Generali";
    let country_name = "Romania";
    let output_file = "generali.json";
    let url = "https://cariere.generali.ro/jobs/search".to_string();
    //let jobs_count = job_count(url).await.unwrap();
    //let mut startrow = 0;
    let mut job_results = Vec::new();
    let mut fetch_jobs_futures = stream::FuturesUnordered::new();
    fetch_jobs_futures.push(fetch_jobs(url, company_name.to_string(), country_name.to_string()));
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

    println!("Parsed {} - Jobs found: {:?} - Took: {}s", company_name, job_results.len(), formatted_seconds);
    let mut file = File::create(output_file)?;
    file.write_all(to_string_pretty(&job_results)?.as_bytes())?;
     
    Ok(())
}
