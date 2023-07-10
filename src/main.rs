use std::env;
use std::fs::{read_dir, remove_file, File};
use std::io::{self, BufRead, Write};
use std::process::exit;

use url::Url;

use nix::sys::signal::{kill, Signal};
use nix::unistd::{getgid, getuid, Pid};

use daemonize::Daemonize;

mod tree;

mod node;
use node::parse_url;

mod daemon;
use daemon::{check_daemon, daemon_server, message_daemon};

const PID_PATH: &str = "/tmp/crawl.pid";
const OUT_PATH: &str = "/tmp/crawl.out";
const ERR_PATH: &str = "/tmp/crawl.err";
const STREAM_PATH: &str = "/tmp/crawl.stream";

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(shell_option) = args.get(1) {
        match shell_option.as_str() {
            "-start" => {
                if let Some(arg) = args.get(2) {
                    let url = parse_url(arg).expect("No valid URL was given");
                    start_option(url);
                } else {
                    println!("Starting daemon");
                    start_crawl();
                };
            }
            "-stop" => {
                let url = parse_url(
                    args.get(2)
                        .expect("Could not find argument in position 2 of the input"),
                )
                .expect("No valid URL was given");
                stop_option(url)
            }
            "-list" => list_option(),
            "-clear" => clear_option(),
            "-kill" => kill_option(),
            "-print" => print_optiony(),
            _ => {
                println!("No recognized option was given");

                exit(-1);
            }
        }
    } else {
        println!("No valid option was specified");
    };
}

fn stop_option(url: Url) {
    if check_daemon() {
        let byte_response = message_daemon("stop".to_string(), Some(url.to_string()));
        println!("{}", String::from_utf8_lossy(&byte_response));
    } else {
        eprintln!("The daemon hasn't been started yet. Please start it")
    }
}

fn start_option(url: Url) {
    let byte_response = message_daemon("start".to_string(), Some(url.to_string()));
    println!("{}", String::from_utf8_lossy(&byte_response));
}

fn start_crawl() {
    if check_daemon() {
        println!("Daemon already running");
    } else {
        let stdout = File::create(OUT_PATH).expect("Couldn't create output file");

        let stderr = File::create(ERR_PATH).expect("Couldn't create error file");

        let daemonize = Daemonize::new()
            .pid_file(PID_PATH)
            .chown_pid_file(true)
            .working_directory("/tmp")
            .umask(0o027) // Set umask, `0o027` by default. This is inverded to chmod in that 0o027 = -rw-r----
            .user(getuid().as_raw())
            .group(getgid().as_raw())
            .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
            .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

        match daemonize.start() {
            Ok(_) => {
                println!("Success, daemonized");
                daemon_server();
            }
            Err(e) => {
                eprintln!("Error, process already running: {}", e);
                exit(-1);
            }
        }
    }
}

fn kill_option() {
    println!("Killing the daemon process and children");
    let file = File::open(PID_PATH).expect("Cant open the pid file, it's likely the program hasn't been started or was stopped already");

    let reader = io::BufReader::new(file);

    let first_line = reader
        .lines()
        .next()
        .expect("Their is nothing left to iterate over")
        .expect("No String was able to be returned in the Result");

    let main_task = Pid::from_raw(
        first_line
            .parse::<i32>()
            .expect("Parse Error on PID resource"),
    );

    if kill(main_task, Signal::SIGINT).is_err() {
        clear_option();
        eprint!("SIGINT signal can't be sent");
    }
}

fn list_option() {
    println!("Listing all scraped sites:");
    let byte_response = message_daemon("list".to_string(), None);
    println!("{}", String::from_utf8_lossy(&byte_response));
}

fn print_optiony() {
    println!("Printing all scraped sites to output.txt");
    let byte_response = message_daemon("list".to_string(), None);
    let mut file = File::create("output.txt").expect("Couldn't create or open output.txt file");

    file.write_all("Site list trees:".as_bytes())
        .expect("Couldn't write the given response");
    file.write_all(&byte_response)
        .expect("Couldn't write the given response");
}

fn clear_option() {
    if let Ok(entries) = read_dir("/tmp/") {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.contains("crawl") {
                    let file_path = entry.path();
                    if let Err(err) = remove_file(&file_path) {
                        eprintln!("Failed to remove file {:?}: {}", file_path, err);
                    } else {
                        println!("File {:?} removed", file_path);
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to read directory.");
    }
}
