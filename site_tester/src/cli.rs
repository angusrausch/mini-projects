use clap::{Arg, ArgAction, ArgMatches, Command};
use site_tester::*;
use std::sync::Arc;
use atty::Stream;
use std::io;
use std::process::exit;

const RED: &str     = "\x1b[31m";
const GREEN: &str   = "\x1b[32m";
const YELLOW: &str  = "\x1b[33m";
const BLUE: &str    = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str    = "\x1b[36m";
const RESET: &str   = "\x1b[0m";
const BOLD: &str    = "\x1b[1m";
const ORANGE: &str  = "\x1b[38;5;202m";

pub struct Config {
    pub url: String,
    pub follow_links: bool,
    pub number: u32,
    pub processes: u32,
    pub method: String,
    pub payload: String,
    pub ignore_ssl: bool,
    pub timeout: u16, // store as milliseconds
    pub verbose: bool,
    pub skip_confirm: bool,
}

impl Config {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
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

pub fn run_cli() {
    
    let matches = get_arguments();

    let config = Config::from_matches(&matches);

    menu(&config);

    let url = Arc::new(config.url.clone());

    let client = get_client(config.timeout, config.ignore_ssl);

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

fn get_arguments() -> ArgMatches {
    Command::new("Async Website Performance Tester")
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
        .arg(
            Arg::new("cli")
                .long("cli")
                .help("Run in CLI mode instead of GUI")
                .action(ArgAction::SetTrue)
        )
        .get_matches()
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