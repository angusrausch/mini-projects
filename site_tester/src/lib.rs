use reqwest::blocking;
use std::{fmt::format, sync::{Arc, Mutex}, time::{Duration, Instant}};
use std::sync::atomic::{AtomicBool, Ordering};

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
            for j in 0..requests_for_this_thread {
                if cancel_flag.load(Ordering::SeqCst) {
                    break;
                }
                let idx = start_idx + j;
                let start = Instant::now();

                let resp = match method {
                    Method::Post => client_arc.post(url_arc.as_str()).send(),
                    Method::Get => client_arc.get(url_arc.as_str()).send(),
                };

                let duration = start.elapsed().as_micros() as u32;
                let mut times_guard = times_arc.lock().unwrap();

                if resp.is_ok() {
                    let response = resp.unwrap();
                    if verbose {
                        out(format!("Status code: {}", response.status()));
                    }
                    times_guard[idx as usize] = duration;
                } else {
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