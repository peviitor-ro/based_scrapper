use anyhow::{Context, Result};

use reqwest::{header, Client};
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use std::time::SystemTime;
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

async fn job_count() -> Result<u64> {
    let url = "https://www.brd.ro/ro/cariere";
    let client = Client::builder()
    .gzip(true)
    .default_headers({
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        headers.insert(header::ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());
        headers.insert(header::ACCEPT,"text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
        headers.insert(header::REFERER, "https://www.google.com/".parse().unwrap());
        headers.insert(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36".parse().unwrap());
        headers
    })
    .build()?;

    let response = client.get(url).send().await.expect("Failed to fetch jobs");
    let html = response.text().await?;

    let document = Html::parse_document(&html);
    let listing_selector = Selector::parse(".col-sm-12.col-md-6").unwrap();
    let jobs_count = document.select(&listing_selector).count().try_into().unwrap();

    Ok(jobs_count)
}

async fn fetch_jobs(url: String, company_name: String, country_name: String) -> Result<Vec<Job>> {
    let client = Client::builder()
    .gzip(true)
    .default_headers({
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        headers.insert(header::ACCEPT_ENCODING, "gzip, deflate, br".parse().unwrap());
        headers.insert(header::ACCEPT,"text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
        headers.insert(header::REFERER, "https://www.google.com/".parse().unwrap());
        headers.insert(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36".parse().unwrap());
        headers
    })
    .build()?;

    let response = client.get(url).send().await?;
    let html = response.text().await?;

    let document = Html::parse_document(&html);
    let listing_selector = Selector::parse(".col-sm-12.col-md-6").unwrap();
    let mut jobs = Vec::new();

    for listing_element in document.select(&listing_selector) {
        let title_selector = Selector::parse(".card-header").unwrap();
        let title_element = listing_element.select(&title_selector).next().unwrap();
        let job_title = title_element.text().collect::<String>().trim().to_owned();

        let city = "Romania".to_string();

        let link_selector = Selector::parse(".button.button-red").unwrap();
        let link_element = listing_element.select(&link_selector).next().unwrap();
        let link = link_element.value().attr("href").unwrap().to_owned();
        let mod_link = format!("{}{}", "https://www.brd.ro", link);
        let mut hasher = XxHash::with_seed(0);
        hasher.write(link.as_bytes());
        let link_hash = hasher.finish();

        jobs.push(Job {
            id: link_hash,
            job_title,
            job_link: mod_link,
            company: company_name.clone(),
            country: country_name.clone(),
            city,
        });
    }
    Ok(jobs)
}

pub async fn scrape() -> Result<()> {
    let start = SystemTime::now();
    let company_name = "BRD";
    let country_name = "Romania";
    let output_file = "brd.json";
    let url = "https://www.brd.ro/cariere/cariere-brd";

    let jobs = fetch_jobs(
        url.to_string(),
        company_name.to_string(),
        country_name.to_string(),
    )
    .await
    .context("Failed to fetch jobs")?;
    let end = SystemTime::now();
    let duration = end.duration_since(start).expect("Time went backwards");

    let elapsed_seconds = duration.as_secs_f64();
    let formatted_seconds = if elapsed_seconds < 1.0 {
        format!("{:.2}", elapsed_seconds)
    } else {
        format!("{:.2}", elapsed_seconds)
    };
    println!(
        "Parsed {} - Jobs found: {} - Took {}s",
        company_name,
        jobs.len(),
        formatted_seconds
    );

    let mut file = File::create(output_file).context("Failed to create output file")?;
    file.write_all(serde_json::to_string_pretty(&jobs)?.as_bytes())
        .context("Failed to write to output file")?;
     
    Ok(())


}
