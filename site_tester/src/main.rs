use atty::Stream;
use clap::{Arg, ArgAction, Command};
use reqwest::{blocking, StatusCode};
use std::{
    io, iter::Skip, process::exit, sync::Arc, thread, time::{Duration, Instant}
};

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
    verbose: bool,
    skip_confirm: bool,
}

impl Config {
    fn from_matches(matches: &clap::ArgMatches) -> Self {
        let timeout_secs = *matches.get_one::<f32>("timeout").unwrap();
        let timeout_ms = (timeout_secs * 1000.0) as u16;
        let url = matches.get_one::<String>("url").unwrap().to_string();
        let url = normalise_url(url, matches.get_flag("force-url"));
        Self {
            url,
            follow_links: matches.get_flag("follow-links"),
            number: *matches.get_one::<u32>("number").unwrap(),
            processes: *matches.get_one::<u32>("processes").unwrap(),
            method: matches.get_one::<String>("type").unwrap().to_string(),
            payload: matches.get_one::<String>("payload").unwrap().to_string(),
            ignore_ssl: matches.get_flag("ignore-ssl"),
            timeout: timeout_ms,
            verbose: matches.get_flag("verbose"),
            skip_confirm: matches.get_flag("skip-confirm"),
        }
    }
}

fn normalise_url(url: String, force_url: bool) -> String {
    if force_url {
        url
    } else if ["https://", "http://"].iter().any(|s| url.starts_with(*s)) {
        url
    } else {
        format!("https://{}", url)
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
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .help("Timeout for each individual request before failing in seconds")
                .default_value("10")
                .value_parser(clap::value_parser!(f32)),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output for requests")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force-url")
                .long("force-url")
                .help("Do not overwrite invalid URL")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("skip-confirm")
                .long("skip-confirm")
                .help("None interactive. Skips confirm step")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let config = Config::from_matches(&matches);

    menu(&config);

    let url = Arc::new(config.url.clone());

    let client = blocking::Client::builder()
        .timeout(Duration::from_millis(config.timeout.into()))
        .danger_accept_invalid_certs(config.ignore_ssl)
        .build()
        .expect("Failed to build client");

    let client = Arc::new(client);

    let times = make_requests(
        url,
        config.number,
        config.processes,
        Arc::clone(&client),
        config.verbose,
    );

    let (average_value, fails, max_time) = get_average(&times);
    println!(
        "{GREEN}Completed a total of {BLUE}{number_requests}{GREEN} requests, \
        with an average time of {BLUE}{:?}{GREEN} for successful requests and \
        {BLUE}{fails}{GREEN} failed requests and a max request time of {BLUE}{:?}{GREEN}",
        average_value,
        max_time,
        number_requests = config.number
    );
}

fn menu(config: &Config) {
    let ascii_banner = format!(
        "{MAGENTA}{BOLD}
     _____ _____ _______ ______ _______ ______  _____ _______ ______ _____  
    / ____|_   _|__   __|  ____|__   __|  ____|/ ____|__   __|  ____|  __ \\ 
   | (___   | |    | |  | |__     | |  | |__  | (___    | |  | |__  | |__) |
    \\___ \\  | |    | |  |  __|    | |  |  __|  \\___ \\   | |  |  __| |  _  / 
    ____) |_| |_   | |  | |____   | |  | |____ ____) |  | |  | |____| | \\ \\ 
   |_____/|_____|  |_|  |______|  |_|  |______|_____/   |_|  |______|_|  \\_\\
{RESET}
"
    );

    println!("{}", ascii_banner);

    let mut message = String::new();
    message.push_str(&format!(
        "{YELLOW}Thank you for using {MAGENTA}Site-Tester{RESET}.\n",
    ));
    message.push_str(&format!(
        "{YELLOW}This application should only be run on websites you have permission from the owner to use.{RESET}\n"
    ));
    message.push_str(&format!(
        "{YELLOW}You have selected website {BOLD}{BLUE}{url}{RESET}{YELLOW} to run on.{RESET}\n",
        url = config.url
    ));
    message.push_str(&format!(
        "{YELLOW}Continuing will make {BOLD}{BLUE}{total_requests}{RESET}{YELLOW} requests \
        using {BOLD}{BLUE}{total_processes}{RESET}{YELLOW} threads.{RESET}\n",
        total_requests = config.number,
        total_processes = config.processes
    ));
    if config.timeout != 10000 {
        message.push_str(&format!(
            "{YELLOW}Using custom timeout value of {BOLD}{BLUE}{timeout}{RESET}{YELLOW} seconds.{RESET}\n",
            timeout = config.timeout / 1000
        ));
    }
    if config.ignore_ssl {
        message.push_str(&format!("{YELLOW}Ignoring any SSL errors{RESET}\n"));
    }

    println!("{}", message);

    if atty::is(Stream::Stdin) && atty::is(Stream::Stdout) && ! config.skip_confirm {
        // Interactive mode
        println!("\n{YELLOW}Press enter to begin or \"exit\" and enter to exit{RESET}");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "" => println!("\n{YELLOW}Starting Program{RESET}"),
            "exit" => {
                println!("\n{GREEN}Exiting Gracefully{RESET}");
                exit(0);
            }
            _ => {
                println!("\n{RED}Unknown Selection{RESET}\n{GREEN}Exiting Gracefully{RESET}");
                exit(0);
            }
        }
    } else {
        // Non-interactive mode (like in your bash script)
        println!("\n{ORANGE}Non-interactive mode detected, starting program automatically{RESET}\n");
    }

}

fn make_requests(
    url: Arc<String>,
    number: u32,
    threads: u32,
    client: Arc<blocking::Client>,
    verbose: bool,
) -> Vec<u32> {
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

fn process(url: &Arc<String>, number: u32, client: Arc<blocking::Client>, verbose: bool,) -> Vec<u32> {
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

fn get_average(times: &Vec<u32>) -> (Duration, u32, Duration) {
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
