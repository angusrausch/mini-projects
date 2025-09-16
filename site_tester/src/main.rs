use atty::Stream;
use clap::{Arg, ArgAction, Command};
use std::{collections::btree_map::Range, io, process::exit, thread, time::Duration, time::Instant};
use reqwest::{blocking, StatusCode};
use std::sync::Arc;

const RED: &str     = "\x1b[31m";
const GREEN: &str   = "\x1b[32m";
const YELLOW: &str  = "\x1b[33m";
const BLUE: &str    = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str    = "\x1b[36m";
const RESET: &str   = "\x1b[0m";
const BOLD: &str    = "\x1b[1m";
const ORANGE: &str  = "\x1b[38;5;202m";

#[derive(Debug)]
struct Config {
    url: String,
    follow_links: bool,
    number: u32,
    processes: u32,
    method: String,
    payload: String,
    ignore_ssl: bool,
    timeout: u16, // store as milliseconds
}

impl Config {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
        let timeout_secs = *matches.get_one::<f32>("timeout").unwrap();
        let timeout_ms = (timeout_secs * 1000.0) as u16;
        Self {
            url: matches.get_one::<String>("url").unwrap().to_string(),
            follow_links: matches.get_flag("follow-links"),
            number: *matches.get_one::<u32>("number").unwrap(),
            processes: *matches.get_one::<u32>("processes").unwrap(),
            method: matches.get_one::<String>("type").unwrap().to_string(),
            payload: matches.get_one::<String>("payload").unwrap().to_string(),
            ignore_ssl: matches.get_flag("ignore-ssl"),
            timeout: timeout_ms,
        }
    }
}

fn main() {
    let matches = Command::new("Async Website Performance Tester")
        .about("Async Website Performance Tester")
        .arg(
            Arg::new("url")
                .long("url")
                .value_name("URL")
                .help("Target URL to test")
                .required(true),
        )
        .arg(
            Arg::new("follow-links")
                .short('f')
                .long("follow-links")
                .help("Follow hyperlinks on the page")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .help("Total number of requests to make")
                .default_value("100")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("processes")
                .short('p')
                .long("processes")
                .help("Number of concurrent workers")
                .default_value("10")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .help("HTTP method to use: get or post")
                .default_value("get")
                .value_parser(["get", "post"]),
        )
        .arg(
            Arg::new("payload")
                .long("payload")
                .help("Raw JSON or path to file with JSON")
                .default_value(""),
        )
        .arg(
            Arg::new("ignore-ssl")
                .long("ignore-ssl")
                .help("Disable SSL check")
                .action(ArgAction::SetFalse), // matches Python's store_false
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .help("Timeout for each individual request before failing in seconds")
                .default_value("10")
                .value_parser(clap::value_parser!(f32)),
        )
        .get_matches();

    let config = Config::from_matches(&matches);

    menu(&config);

    let url = Arc::new(config.url);
    let times = make_requests(url, config.number, config.processes, config.ignore_ssl, config.timeout);

    let (average, fails): (u32, u32) = get_average(&times);
    println!("{GREEN}Completed a total of {BLUE}{number_requests}{GREEN} requests, with an average time of {BLUE}{average}{GREEN} for successful requests and {BLUE}{fails}{GREEN} failed requests", 
        number_requests = config.number);
}

fn menu(config: &Config) {
    let ascii_banner = format!(
        "{magenta}{bold}
     _____ _____ _______ ______ _______ ______  _____ _______ ______ _____  
    / ____|_   _|__   __|  ____|__   __|  ____|/ ____|__   __|  ____|  __ \\ 
   | (___   | |    | |  | |__     | |  | |__  | (___    | |  | |__  | |__) |
    \\___ \\  | |    | |  |  __|    | |  |  __|  \\___ \\   | |  |  __| |  _  / 
    ____) |_| |_   | |  | |____   | |  | |____ ____) |  | |  | |____| | \\ \\ 
   |_____/|_____|  |_|  |______|  |_|  |______|_____/   |_|  |______|_|  \\_\\
{reset}
",
        magenta = MAGENTA,
        bold = BOLD,
        reset = RESET
    );

    println!("{}", ascii_banner);

    let mut message = String::new();
    message.push_str(&format!(
        "{yellow}Thank you for using {magenta}Site-Tester{reset}.\n",
        yellow = YELLOW,
        magenta = MAGENTA,
        reset = RESET
    ));
    message.push_str(&format!(
        "{yellow}This application should only be run on websites you have permission from the owner to use.{reset}\n",
        yellow = YELLOW,
        reset = RESET
    ));
    // Normalise URL here
    message.push_str(&format!(
        "{yellow}You have selected website {bold}{blue}{url}{reset}{yellow} to run on.{reset}\n",
        yellow = YELLOW,
        bold = BOLD,
        blue = BLUE,
        url = config.url,
        reset = RESET
    ));
    message.push_str(&format!(
        "{yellow}Continuing will make {bold}{blue}{total_requests}{reset}{yellow} requests to the server using {bold}{blue}{total_processes}{reset}{yellow} threads.{reset}\n",
        yellow = YELLOW,
        bold = BOLD,
        blue = BLUE,
        total_requests = config.number,
        total_processes = config.processes,
        reset = RESET
    ));
    // Follow links randomly (probably won't implement on this version)
    // Custom timeout value
    // Ignore SSL option

    println!("{}", message);

    // Unsure if this works
    if atty::is(Stream::Stdin) {
        println!("\n{YELLOW}Press enter to begin or \"exit\" and enter to exit{RESET}");
        
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if input.trim() == "" {
            println!("\n{YELLOW}Starting Program{RESET}");
        } else if input.trim() == "exit" {
            println!("\n{GREEN}Exiting Gracefully{RESET}");
            exit(0)
        } else {
            println!("\n{RED}Unknown Selection{RESET}\n{GREEN}Exiting Gracefully{RESET}");
            exit(0)
        }
    } else {
        println!("\n{ORANGE}Non-interactive mode detected{RESET}\n",
            ORANGE = ORANGE, RESET = RESET);
    }
}

fn make_get_request(client: &blocking::Client, url: &str, ignore_cert: bool, timeout: u16) -> u32 {
    let start = Instant::now();

    let resp = client.get(url).send();
    let duration = start.elapsed().as_micros() as u32;

    match resp {
        Ok(response) => {
            let status: u16 = response.status().as_u16();
            // println!("Status code: {}", status);
            duration
        }
        Err(e) => {
            // println!("Request failed: {}", e);
            0
        }
    }
}

fn make_requests(url: Arc<String>, number: u32, threads: u32, ignore_cert: bool, timeout: u16) -> Vec<u32> {
    let mut times: Vec<u32> = vec![0; number as usize];
    let number_per_thread = number / threads;
    let remainder = number % threads;
    let mut children = vec![];

    for i in 0..threads {
        let url_arc = Arc::clone(&url);
        let requests_for_this_thread = number_per_thread + if i < remainder { 1 } else { 0 };
        children.push(thread::spawn(move || {
            process(&url_arc, requests_for_this_thread, ignore_cert, timeout)
        }));
    }

    // Collect results from threads
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

fn process(url: &Arc<String>, number: u32, ignore_cert: bool, timeout: u16) -> Vec<u32> {
    let mut thread_times: Vec<u32> = vec![0; number as usize];
    let client = blocking::Client::builder()
        .timeout(Duration::from_millis(timeout.into()))
        .danger_accept_invalid_certs(!ignore_cert)
        .build()
        .expect("Failed to build client");

    for index in 0..number {
        thread_times[index as usize] = make_get_request(&client, url.as_str(), ignore_cert, timeout);
    }

    thread_times
}

fn get_average(times: &Vec<u32>) -> (u32, u32) {
    let mut total: u32 = 0;
    let mut successes: u32 = 0;
    let mut fails: u32 = 0;
    for time in times {
        if *time == 0 {
            fails += 1;
        } else {
            successes += 1;
            total += *time;
        }
    }
    let average: u32 = total / successes;

    return (average, fails);
}