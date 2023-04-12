use anyhow::{Context, Result};

use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use std::time::SystemTime;
use titlecase::titlecase;
use twox_hash::XxHash;
use unidecode::unidecode;
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
    let listing_selector = Selector::parse(".box-listing-job").unwrap();
    let mut jobs = Vec::new();

    for listing_element in document.select(&listing_selector) {
        let title_selector = Selector::parse(".nume-listing-job").unwrap();
        let title_element = listing_element.select(&title_selector).next().unwrap();
        let job_title = title_element.text().collect::<String>().trim().to_owned();

        let location_selector = Selector::parse(".locatie-job").unwrap();
        let location_element = listing_element.select(&location_selector).next().unwrap();
        let city = titlecase(
            &location_element
                .text()
                .collect::<String>()
                .trim()
                .to_owned(),
        );

        let link = listing_element.value().attr("href").unwrap().to_owned();

        let mut hasher = XxHash::with_seed(0);
        hasher.write(link.as_bytes());
        let link_hash = hasher.finish();

        jobs.push(Job {
            id: link_hash,
            job_title,
            job_link: link.replace(" ", ""),
            company: company_name.clone(),
            country: country_name.clone(),
            city: unidecode(&city),
        });
    }
    Ok(jobs)
}

pub async fn scrape() -> Result<()> {
    let start = SystemTime::now();
    let company_name = "Autonom";
    let country_name = "Romania";
    let output_file = "autonom.json";
    let url = "https://www.autonom.ro/cariere";

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
