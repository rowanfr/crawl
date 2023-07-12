use std::collections::{HashSet, VecDeque};
use std::error::Error;

use crate::tree::{SiteTree, SubSites};

use reqwest::{header, Client};

use select::document::Document;
use select::predicate::Name;

use url::{ParseError, Url};

pub fn parse_url(shell_arg: &String) -> Result<Url, ParseError> {
    if let Ok(url) = Url::parse(shell_arg) {
        Ok(url)
    } else if let Ok(url) = Url::parse(&("http://".to_string() + shell_arg)) {
        Ok(url)
    } else if let Ok(url) = Url::parse(&("https://".to_string() + shell_arg)) {
        Ok(url)
    } else {
        Err(ParseError::IdnaError)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_url, tree_url_get};
    use crate::tree::{SiteTree, SubSites};
    use std::collections::{HashSet, VecDeque};
    use url::Url;

    #[test]
    fn test_parse_url() {
        let url_test = String::from("www.example.com");
        let parsed_url = parse_url(&url_test).expect("Couldn't parse the given URL");
        println!("{}", parsed_url);
        assert_eq!(
            Url::parse("http://www.example.com").expect("Couldn't parse the given URL"),
            parsed_url
        );
    }

    #[test]
    fn test_tree_url_get() {
        let site_tree = SiteTree {
            current_site: Url::parse("http://www.example.com").unwrap(),
            sub_sites: SubSites::List(vec![SiteTree {
                current_site: Url::parse("https://www.iana.org/domains/example").unwrap(),
                sub_sites: SubSites::Nil,
            }]),
        };
        let url = Url::parse("http://www.example.com").expect("Couldn't parse the given URL");
        let mut node = SiteTree {
            current_site: url.clone(),
            sub_sites: SubSites::Nil,
        };
        let mut site_set: HashSet<String> = HashSet::new();
        let mut job_queue: VecDeque<&mut SiteTree> = VecDeque::new();

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                {
                    tree_url_get(
                        &mut node,
                        url.domain()
                            .expect("The Domain was unable to be extracted from the url"),
                        &mut site_set,
                        &mut job_queue,
                    )
                    .await
                    .expect("Wasn't able to parse the tree URL");
                }
            });
        println!("\nJob queue iterator: ");
        for value in job_queue.iter() {
            print!("{}", value);
        }
        println!();
        println!("Generated node:\n{}", node);
        println!("Hash set iterator: ");
        for value in site_set.iter() {
            println!("{}", value);
        }

        println!();
        assert_eq!(site_tree, node);
    }

    #[test]
    fn test_long_queue() {
        let url = Url::parse("https://spideroak.com").expect("Couldn't parse the given URL");
        let mut node = SiteTree {
            current_site: url.clone(),
            sub_sites: SubSites::Nil,
        };
        let mut site_set: HashSet<String> = HashSet::new();
        let mut job_queue: VecDeque<&mut SiteTree> = VecDeque::new();

        job_queue.push_back(&mut node);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                {
                    while let Some(task) = job_queue.pop_front() {
                        print!("Current site being scanned: {}", task);
                        let tree_result = tree_url_get(
                            &mut (*task),
                            url.domain()
                                .expect("The Domain was unable to be extracted from the url"),
                            &mut site_set,
                            &mut job_queue,
                        )
                        .await;
                        if let Err(e) = tree_result {
                            eprintln!("There was an error in parsing the URL or scraping the site. The error is: {}",&*e)
                        }
                    }
                }
            });
        println!("\nJob queue iterator: ");
        for value in job_queue.iter() {
            print!("{}", value);
        }
        println!();
        println!("Generated node:\n{}", node);
        println!("Hash set iterator: ");
        for value in site_set.iter() {
            println!("{}", value);
        }
    }

    #[test]
    fn test_empty_site() {
        let url = Url::parse("http://itcorp.com/").expect("Couldn't parse the given URL");
        let mut node = SiteTree {
            current_site: url.clone(),
            sub_sites: SubSites::Nil,
        };
        let mut site_set: HashSet<String> = HashSet::new();
        let mut job_queue: VecDeque<&mut SiteTree> = VecDeque::new();

        job_queue.push_back(&mut node);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                {
                    while let Some(task) = job_queue.pop_front() {
                        tree_url_get(
                            &mut (*task),
                            url.domain()
                                .expect("The Domain was unable to be extracted from the url"),
                            &mut site_set,
                            &mut job_queue,
                        )
                        .await
                        .expect("Unable to parse the tree URL");
                    }
                }
            });
        println!("\nJob queue iterator: ");
        for value in job_queue.iter() {
            print!("{}", value);
        }
        println!();
        println!("Generated node:\n{}", node);
        println!("Hash set iterator: ");
        for value in site_set.iter() {
            println!("{}", value);
        }
    }

    #[test]
    fn test_large_file() {
        let url = Url::parse("https://spideroak.com/release/crossclave/osx")
            .expect("Couldn't parse the given URL");
        let mut node = SiteTree {
            current_site: url.clone(),
            sub_sites: SubSites::Nil,
        };
        let mut site_set: HashSet<String> = HashSet::new();
        let mut job_queue: VecDeque<&mut SiteTree> = VecDeque::new();

        job_queue.push_back(&mut node);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                {
                    while let Some(task) = job_queue.pop_front() {
                        print!("Current site being scanned: {}", task);
                        tree_url_get(
                            &mut (*task),
                            url.domain()
                                .expect("The Domain was unable to be extracted from the url"),
                            &mut site_set,
                            &mut job_queue,
                        )
                        .await
                        .expect("Unable to parse the tree URL");
                    }
                }
            });
        println!("\nJob queue iterator: ");
        for value in job_queue.iter() {
            print!("{}", value);
        }
        println!();
        println!("Generated node:\n{}", node);
        println!("Hash set iterator: ");
        for value in site_set.iter() {
            println!("{}", value);
        }
    }
}

pub async fn tree_url_get<'a>(
    node: &'a mut SiteTree,
    domain: &str,
    site_set: &mut HashSet<String>,
    job_queue: &mut VecDeque<&'a mut SiteTree>,
) -> Result<(), Box<dyn Error>> {
    //Check if the domain can be determined
    let node_domain: &str = if let Some(node_domain) = node.current_site.domain() {
        node_domain
    } else {
        return Ok(());
    };
    //Check if the domain already exists
    if node_domain != domain || site_set.contains(&node.current_site.to_string()) {
        return Ok(());
    } else {
        site_set.insert(node.current_site.to_string());
    }

    let client = Client::new();

    //Get the header from the html request to determine necessary featrues about the pages
    //This can be maliciously poisened to cause a stack overflow
    let head_req = client.head(node.current_site.clone()).send().await?;
    if let Some(content_length) = head_req.headers().get(header::CONTENT_LENGTH) {
        println!("{:?}", content_length);
    };
    let content_type = head_req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_lowercase());

    let html_req: String = if let Some(content_type) = content_type {
        if content_type.starts_with("text/html") {
            // If the response is HTML, proceed with fetching and parsing the text

            client
                .get(node.current_site.clone())
                .send()
                .await?
                .text()
                .await?

            // Continue with the rest of your code to parse the HTML response
        } else {
            // Handle non-HTML response, if necessary
            return Ok(());
        }
    } else {
        // Handle missing Content-Type header, if necessary
        return Ok(());
    }; //Can still stack overflow due to a poisoned html request

    let mut sub_sites: Vec<SiteTree> = Vec::new();

    let mut href_errors: Vec<Result<(), Box<dyn Error>>> = Vec::new();

    let mut local_duplicate_set: HashSet<Url> = HashSet::new();

    //Filter out the links from the html code
    Document::from(html_req.as_str())
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .try_for_each(|href| {
            if let Ok(url) = Url::parse(href) {
                if local_duplicate_set.contains(&url) {
                } else {
                    //One can use this region to modify duplicate nodes with an identifier
                    local_duplicate_set.insert(url.clone());
                    let site_tree = SiteTree {
                        current_site: url,
                        sub_sites: SubSites::Nil,
                    };
                    sub_sites.push(site_tree);
                    href_errors.push(Ok(()));
                }
            } else if let Ok(url) = {
                let sub_site_host = node.current_site.clone();
                sub_site_host.join(href)
            } {
                if local_duplicate_set.contains(&url) {
                } else {
                    local_duplicate_set.insert(url.clone());

                    let site_tree = SiteTree {
                        current_site: url,
                        sub_sites: SubSites::Nil,
                    };
                    sub_sites.push(site_tree);
                    // Handle the case when URL parsing fails
                    // Error will cause the rest to not propogate
                    href_errors.push(Ok(()))
                };
            } else {
                href_errors.push(Err(Box::new(ParseError::IdnaError)));
            }
            Result::<(), ()>::Ok(())
        })
        .expect("Couldn't handle the result from parsing");

    //Attach the links gathered on the page to the passed in node and add references to all sub-nodes to the queue
    node.sub_sites = SubSites::List(sub_sites);
    if let SubSites::List(sub_sites_list) = &mut node.sub_sites {
        for site_tree in sub_sites_list.iter_mut() {
            job_queue.push_back(site_tree);
        }
    }

    //Check for if all values are errors. If one value isn't it still gets passed without error but if all are this panics with the expectation that something is wrong
    let (oks, errs): (Vec<_>, Vec<_>) = href_errors.into_iter().partition(Result::is_ok);

    if !oks.is_empty() {
        //The vector contains at least one Ok value
        Ok(())
    } else {
        //The vector does not contain any Ok values
        if !errs.is_empty() {
            //The vector contains at least one Err value
            Err(Box::new(ParseError::IdnaError))
        } else {
            //The vector does not contain any Err values
            Ok(())
        }
    }
}
