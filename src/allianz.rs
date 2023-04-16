
use futures::{stream, StreamExt};
use regex::Regex;
use reqwest::Error;
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::HashSet;
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

async fn fetch_jobs(
    url: String,
    company_name: String,
    country_name: String,
) -> Result<Vec<Job>, Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);
    let selector =
        Selector::parse(".col-md-12.sub-section.sub-section-desktop.hidden-xs.hidden-sm").unwrap();
    let mut result = Vec::new();

    for job_element in document.select(&selector) {
        let title_selector = Selector::parse(".tiletitle a").unwrap();
        let title_element = job_element.select(&title_selector).next().unwrap();
        let title = title_element.text().collect::<String>().trim().to_string();
        let url = title_element.value().attr("href").unwrap().to_string();
        let city = country_name.to_string();
        //in caz ca scapa ceva dubios in array
        let search_chars = ['ç', 'é', 'û', 'ê', 'ô', 'û', 'ê', 'ô', 'û', 'ë', 'ï', 'ü'];
        if !search_chars.iter().any(|&c| title.contains(c)) {
            result.push(Job {
                id: url
                    .split("/")
                    .filter(|s| !s.is_empty())
                    .last()
                    .unwrap_or("error")
                    .to_string(),
                job_title: title,
                job_link: format!("https://careers.allianz.com{}", url),
                company: company_name.to_string(),
                country: country_name.to_string(),
                city: unidecode(&city),
            });
        }
    }

    Ok(result)
}

fn remove_duplicates(jobs: Vec<Job>) -> Vec<Job> {
    let mut links = HashSet::new();
    let mut results = Vec::new();
    for job in jobs {
        if links.insert(job.job_link.clone()) {
            results.push(job);
        }
    }
    results
}
async fn job_count() -> Result<u64, Error> {
    let url = "https://careers.allianz.com/search/?searchby=location&createNewAlert=false&q=&locationsearch=Romania&optionsFacetsDD_department=&optionsFacetsDD_shifttype=&optionsFacetsDD_customfield3=&optionsFacetsDD_customfield2=&optionsFacetsDD_facility=&optionsFacetsDD_customfield4=&inputSearchValue=Romania&quatFlag=false";
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);
    let regex = Regex::new(r#"jobRecordsFound:\s*parseInt\("(\d+)"\)"#).unwrap();
    let mut jobs_count: u32 = 0;
    for node in document.select(&Selector::parse("script").unwrap()) {
        if let Some(_script) = node.value().attr("src") {
            continue;
        }
        if let Some(contents) = node.text().next() {
            if let Some(captures) = regex.captures(contents) {
                jobs_count = captures[1].parse().unwrap();
                break;
            }
        }
    }
    Ok(jobs_count.into())
}

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now();
    let company_name = "Allianz";
    let country_name = "Romania";
    let output_file = "allianz.json";
    let jobs_count = job_count().await.unwrap();
    let mut startrow = 0;
    let mut job_results = Vec::new();
    let mut fetch_jobs_futures = stream::FuturesUnordered::new();
    while startrow < jobs_count {
        let search_url = format!("https://careers.allianz.com/tile-search-results?q=&locationsearch=Romania&searchby=location&d=100&startrow={}", startrow);
        fetch_jobs_futures.push(fetch_jobs(
            search_url,
            company_name.to_string(),
            country_name.to_string(),
        ));
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
    let unique_jobs = remove_duplicates(job_results);

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
        unique_jobs.len(),
        formatted_seconds
    );
    let mut file = File::create(output_file)?;
    file.write_all(to_string_pretty(&unique_jobs)?.as_bytes())?;
    Ok(())
}
