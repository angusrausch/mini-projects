use reqwest::blocking;
use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};
use std::sync::atomic::{AtomicBool, Ordering};
use scraper::{Html, Selector};
use rand::prelude::*;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get = 1,
    Post = 2,
}

impl Method {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "post" => Method::Post,
            _ => Method::Get,
        }
    }
}

pub fn normalise_url(url: String, force_url: bool) -> String {
    if force_url {
        url
    } else if ["https://", "http://"].iter().any(|s| url.starts_with(*s)) {
        url
    } else {
        format!("https://{}", url)
    }
}

pub fn get_client(timeout: u16, ignore_ssl: bool) -> Arc<blocking::Client> {
    let client = blocking::Client::builder()
        .timeout(Duration::from_millis(timeout.into()))
        .danger_accept_invalid_certs(ignore_ssl)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to build client");

    Arc::new(client)
}

pub fn make_requests<O, E>(
    url: Arc<String>,
    number: u32,
    threads: u32,
    client: Arc<blocking::Client>,
    verbose: bool,
    output: (O, E),
    times: Arc<Mutex<Vec<u32>>>,
    cancel_flag: Arc<AtomicBool>,
    method: Method,
    follow_links: bool
)
where
    O: Fn(String) + Send + Sync + 'static + Clone,
    E: Fn(String) + Send + Sync + 'static + Clone,
{
    let number_per_thread = number / threads;
    let remainder = number % threads;

    for i in 0..threads {
        let url_arc = Arc::clone(&url);
        let client_arc = Arc::clone(&client);
        let output_clone = output.clone();
        let times_arc = Arc::clone(&times);
        let cancel_flag = Arc::clone(&cancel_flag);
        let requests_for_this_thread = number_per_thread + if i < remainder { 1 } else { 0 };
        let start_idx = i * number_per_thread + std::cmp::min(i, remainder);

        std::thread::spawn(move || {
            let (out, err) = output_clone;
            let mut thread_url = Arc::clone(&url_arc);
            for j in 0..requests_for_this_thread {
                out(thread_url.to_string());
                if cancel_flag.load(Ordering::SeqCst) {
                    break;
                }
                let idx = start_idx + j;
                let start = Instant::now();

                let resp = match method {
                    Method::Post => client_arc.post(thread_url.as_str()).send(),
                    Method::Get => client_arc.get(thread_url.as_str()).send(),
                };

                let duration = start.elapsed().as_micros() as u32;
                let mut times_guard = times_arc.lock().unwrap();

                if resp.is_ok() {
                    let response = resp.unwrap();
                    let status = response.status();
                    let body = response
                        .text()
                        .unwrap_or_else(|e| format!("Failed to read body: {:?}", e));
                    if status.as_u16() == 509 {
                        out("Website bandwidth limit reached".to_string());
                    }
                    if follow_links {
                        if let Some(next) = get_href(body, &url_arc, &thread_url) {
                            thread_url = Arc::new(next);
                        } else {
                            thread_url = Arc::clone(&url_arc);
                        }
                    }
                    if verbose {
                        out(format!("Status code: {}", status));
                    }
                    times_guard[idx as usize] = duration;
                } else {
                    if follow_links {
                        thread_url = Arc::clone(&url_arc);
                    }
                    if verbose {
                        let e = resp.unwrap_err();
                        let err_msg = format!(
                            "Request failed at idx {}: {:?}\nURL: {}\nThread: {}\nError: {:?}",
                            idx, e, url_arc, i, e
                        );
                        err(err_msg);
                    } else {
                        let err_msg = format!(
                            "Failed Request Number: {} ", idx
                        );
                        err(err_msg);
                    }
                    times_guard[idx as usize] = u32::MAX; 
                }
            }
        });
    }
}

pub fn get_average(times: &Vec<u32>) -> (Duration, u32, Duration) {
    let mut total: u32 = 0;
    let mut successes: u32 = 0;
    let mut fails: u32 = 0;
    let mut max: u32 = 0;
    for time in times {
        if *time == 0 || *time == u32::MAX {
            fails += 1;
        } else {
            successes += 1;
            total += *time;
            max = max.max(*time);
        }
    }

    let average: u32 = if successes > 0 { total / successes } else { 0 };
    let average_value = Duration::from_micros(average as u64);
    let max = Duration::from_micros(max as u64);

    (average_value, fails, max)
}

fn get_href(body: String, base_url: &Arc<String>, thread_url: &Arc<String>) -> Option<String> {
    let base = Url::parse(&base_url).unwrap();

    let html = Html::parse_document(&body);
    let href_selector = Selector::parse("a").unwrap();
    
    let mut hrefs = Vec::new();


for element in html.select(&href_selector) {
    if let Some(href) = element.value().attr("href") {
        if href.starts_with("mailto:") {
            continue;
        } else if href == thread_url.as_str() {
            continue;
        }

        let parsed = if let Ok(url) = Url::parse(href) {
            url
        } else if let Ok(joined) = base.join(href) {
            joined
        } else {
            continue;
        };

        if parsed.domain() == base.domain() {
            hrefs.push(parsed.to_string());
        }
    }
}

    let mut rng = rand::rng(); 
    
    hrefs.choose(&mut rng).cloned() 
}