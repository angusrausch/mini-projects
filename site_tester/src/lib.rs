use reqwest::{blocking, StatusCode};
use std::{sync::Arc, thread, time::{Duration, Instant}};

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
        .build()
        .expect("Failed to build client");

    Arc::new(client)
}

pub fn make_requests(url: Arc<String>, number: u32, threads: u32, client: Arc<blocking::Client>, verbose: bool) -> Vec<u32> {
    let mut times: Vec<u32> = vec![0; number as usize];
    let number_per_thread = number / threads;
    let remainder = number % threads;
    let mut children = vec![];

    for i in 0..threads {
        let url_arc = Arc::clone(&url);
        let client_arc = Arc::clone(&client);
        let requests_for_this_thread = number_per_thread + if i < remainder { 1 } else { 0 };
        children.push(thread::spawn(move || {
            process(&url_arc, requests_for_this_thread, client_arc, verbose)
        }));
    }

    let mut idx = 0;
    for child in children {
        let thread_times = child.join().expect("Thread panicked");
        for time in thread_times {
            if idx < times.len() {
                times[idx] = time;
                idx += 1;
            }
        }
    }

    times
}

pub fn process(url: &Arc<String>, number: u32, client: Arc<blocking::Client>, verbose: bool,) -> Vec<u32> {
    let mut thread_times: Vec<u32> = vec![0; number as usize];

    for index in 0..number {
        let start = Instant::now();
        let resp = client.get(url.as_str()).send();
        let duration = start.elapsed().as_micros() as u32;

        thread_times[index as usize] = match resp {
            Ok(response) => {
                if verbose && response.status() != StatusCode::OK {
                    println!("Status code: {}", response.status());
                }
                duration
            }
            Err(e) => {
                if verbose {
                    println!("Request failed: {}", e);
                }
                0
            }
        };
    }

    thread_times
}

pub fn get_average(times: &Vec<u32>) -> (Duration, u32, Duration) {
    let mut total: u32 = 0;
    let mut successes: u32 = 0;
    let mut fails: u32 = 0;
    let mut max: u32 = 0;
    for time in times {
        if *time == 0 {
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