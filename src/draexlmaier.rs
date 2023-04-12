use serde_json::to_string_pretty;
use anyhow::{Result,Error};
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use titlecase::titlecase;
use unidecode::unidecode;
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
    let listing_selector = Selector::parse(".data-row").unwrap();
    let mut jobs = Vec::new();

    for listing_element in document.select(&listing_selector) {
        let title_selector = Selector::parse(".jobTitle-link").unwrap();
        let title_element = listing_element.select(&title_selector).next().unwrap();
        let job_title = title_element.text().collect::<String>().trim().to_owned();

        let location_selector = Selector::parse(".jobLocation").unwrap();
        let location_element = listing_element.select(&location_selector).next().unwrap();
        let location = location_element.text().collect::<String>().trim().to_owned();
        let city = location.split(',').next().unwrap().trim().to_owned();

        let link = title_element.value().attr("href").unwrap().to_owned();
        let mod_link = format!("{}{}","https://d-career.org", link); 
        let mut hasher = XxHash::with_seed(0);
        hasher.write(link.as_bytes());
        let link_hash = hasher.finish();


       jobs.push(Job {
            id: link_hash,
            job_title,
            job_link: mod_link,
            company: company_name.clone(),
            country: country_name.clone(),
            city: unidecode(&titlecase(&city)),
        });
    }
    Ok(jobs)
}

async fn job_count(url: String) -> Result<u32,Error> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36")
        .build()?;
    let response = client.get(url).send().await?;
    let html = response.text().await?;

    let document = Html::parse_document(&html);

    let selector = Selector::parse(".paginationLabel b").unwrap();
    let b_elements: Vec<_> = document.select(&selector).collect();
    let count = b_elements.get(1).unwrap().inner_html().parse::<u32>()?;
    Ok(count)
}


pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let company_name = "Draexlmaier";
    let country_name = "Romania";
    let output_file = "dralexmaier.json";
    let url = "https://d-career.org/Draexlmaier/go/DR%C3%84XLMAIER-Job-Opportunities-in-Romania-%28Romanian%29/4196801/0/?q=&sortColumn=referencedate&sortDirection=desc".to_string();
    let jobs_count = job_count(url).await.unwrap();
    let mut startrow = 0;
    let mut job_results = Vec::new();
    let mut fetch_jobs_futures = stream::FuturesUnordered::new();
    while startrow < jobs_count {
        let search_url = format!("https://d-career.org/Draexlmaier/go/DR%C3%84XLMAIER-Job-Opportunities-in-Romania-%28Romanian%29/4196801/{}/?q=&sortColumn=referencedate&sortDirection=desc", startrow);
        fetch_jobs_futures.push(fetch_jobs(search_url, company_name.to_string(), country_name.to_string()));
        startrow += 25;
    }
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
