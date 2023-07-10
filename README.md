# crawl
The application consists of a command line client and a local service (daemon) which performs the actual web crawling. The communication between client and server should use a form of IPC mechanism. For each URL, the Web Crawler creates a tree of links with the root of the tree being the root URL.
