use libc;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{self, BufRead};
use std::process::exit;

use crate::node::{parse_url, tree_url_get};
use crate::tree::{SiteTree, SubSites};
use crate::{clear_option, kill_option, PID_PATH, STREAM_PATH};

use nix::sys::signal::{kill, sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal, SIGINT};
use nix::unistd::Pid;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

extern "C" fn handle_sigint(_: libc::c_int, _: *mut libc::siginfo_t, _: *mut libc::c_void) {
    clear_option();
    exit(0);
}

pub fn check_daemon() -> bool {
    let pid_file = File::open(PID_PATH).is_ok();

    let daemon_active = if pid_file {
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
        kill(main_task, None).is_ok()
    } else {
        false
    };

    if pid_file && daemon_active {
        true
    } else if pid_file || daemon_active {
        kill_option();
        panic!("An unknown error occured due to a mismatch of the PID file being available but the daemon not being active");
    } else {
        false
    }
}

pub fn message_daemon(command: String, website: Option<String>) -> Vec<u8> {
    #[allow(unused_assignments)]
    let mut message_to_server = String::new();
    if let Some(website) = website {
        message_to_server = format!("{} {}", command.as_str(), website.as_str())
    } else {
        message_to_server = format!("{}", command.as_str())
    }

    let mut byte_response = Vec::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Connect to the UnixListener
            let stream = UnixStream::connect(STREAM_PATH)
                .await
                .expect("Couldn't connect to the unix stream");
            byte_response = handle_server(stream, message_to_server).await;

            Result::<(), ()>::Ok(())
        })
        .expect("tokio runtime failed");
    byte_response
}

async fn handle_server(mut stream: UnixStream, command: String) -> Vec<u8> {
    stream
        .write_all(command.as_bytes())
        .await
        .expect("Couldn't stream the given command as bytes");

    // Write data to the server
    let mut buffer = Vec::new();
    // Read the server's response
    loop {
        let mut chunk = vec![0; 1024];
        let bytes_read = stream
            .read(&mut chunk)
            .await
            .expect("Cannot read information from the socket");
        if bytes_read == 0 {
            // No more data to read, exit the loop
            break;
        }

        // Accumulate the received data
        buffer.extend_from_slice(&chunk[..bytes_read]);
    }
    buffer
}

pub fn daemon_server() {
    unsafe {
        let mut sigset = SigSet::empty();
        sigset.add(Signal::SIGINT);
        let sig_action = SigAction::new(
            SigHandler::SigAction(handle_sigint),
            SaFlags::empty(),
            sigset,
        );
        sigaction(SIGINT, &sig_action).expect("SigAction could not be set");
    }
    let mut processes: HashMap<String, tokio::task::JoinHandle<SiteTree>> = HashMap::new();
    let mut completed: HashMap<String, String> = HashMap::new();

    tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(
        async {
            let listener = UnixListener::bind(STREAM_PATH)
                .expect("Bind the unix listener to the path");
            loop {
                match listener.accept().await {
                    Ok((mut stream, _addr)) => {
                        let mut buffer = [0; 1024];
                        let bytes_read = stream.read(&mut buffer).await.unwrap();
                        // Process the received command
                        let received_data =
                            String::from_utf8_lossy(&buffer[..bytes_read]);
                        let mut parts = received_data.trim().splitn(2, ' ');
                        let command = parts.next().unwrap_or("");
                        let argument = if let Some(argument) = parts.next() {
                            argument
                        } else {
                            ""
                        };
                        let mut response = String::new();
                        match command {
                            "stop" => {
                                println!(
                                    "Stop command received with argument: {}",
                                    argument
                                );
                                if processes.contains_key(argument){
                                    response = String::from("Stopped scraping ") + argument.clone();
                                    processes.get(argument).expect("Couldn't get value from hash map").abort();
                                    processes.remove(argument);
                                } else {
                                    response = String::from("The daemon is not scraping ") + argument.clone();
                                }
                            }
                            "start" => {
                                println!(
                                    "Start command received with argument: {}",
                                    argument
                                );
                                #[allow(clippy::map_entry)]
                                if !processes.contains_key(&argument.to_string()) && !completed.contains_key(&argument.to_string()) {
                                    response = String::from("Started scraping ")
                                    + argument.clone();
                                let st_url = if let Ok(st_url) =
                                    parse_url(&argument.to_string())
                                {
                                    st_url
                                } else {
                                    response =
                                        String::from("Failed to get valid URL");
                                        stream
                                            .write_all(response.as_bytes())
                                            .await
                                            .unwrap();
                                    continue;
                                };
                                let background_process = tokio::spawn(async move {

                                    let mut node = SiteTree {
                                        current_site: st_url.clone(),
                                        sub_sites: SubSites::Nil,
                                    };
                                    let mut site_set: HashSet<String> = HashSet::new();
                                    let mut job_queue: VecDeque<&mut SiteTree> = VecDeque::new();
                                    job_queue.push_back(&mut node);
                                    while let Some(task) = job_queue.pop_front() {
                                        print!("Current site being scanned: {}", task);
                                        tree_url_get(
                                            &mut (*task),
                                            st_url.clone().domain()
                                                .expect("The Domain was unable to be extracted from the url"),
                                            &mut site_set,
                                            &mut job_queue,
                                        )
                                        .await
                                        .expect("Unable to parse the tree URL");
                                    }
                                    node
                                });
                                processes.insert(argument.to_string(), background_process);
                                } else {
                                    response = String::from("Already scraping or scraped ")
                                    + argument.clone();
                                }
                            }
                            "list" => {
                                println!("List command received");
                                for (site,job_handle) in processes.iter_mut() {
                                    if job_handle.is_finished() {
                                            match job_handle.await{
                                                Ok(full_tree) => {
                                                    completed.insert(site.clone(), format!("{}",full_tree));
                                                },
                                                Err(_) => {
                                                    println!("The tree for {} didn't complete properly",site)
                                                }
                                            };
                                    } else {
                                        //Handle partial job
                                        response = site.clone() + " is still being processed";
                                    }
                                }
                                //Slightly inefficiant to try and remove values every time but it's a minor computation once every call vs another data structure
                                for (site, tree) in completed.iter() {
                                    processes.remove(site);
                                    response = format!("{}\n{}",response,tree);
                                }
                            }
                            _ => {
                                println!("Unknown command");
                                response = String::from("Unknown command");
                            }
                        }
                        stream
                            .write_all(response.as_bytes())
                            .await
                            .unwrap();

                        // Write a response back to the client
                    }
                    Err(e) => {
                        panic!("Connection failed due to {}", e);
                    }
                }
            }
            Result::<(), Box<dyn std::error::Error>>::Ok(())
        },)
    .expect("Could not create a tokio runtime environment");
}
