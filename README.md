# crawl
The application consists of a command line client and a local service (daemon) which performs the actual web crawling. The communication between client and server uses a form of IPC mechanism. For each URL, the Web Crawler creates a tree of links with the root of the tree being the root URL.

The commands for the application are as follows:
- -start
        This starts the daemon if one doesn't exist
- -start url
        This starts the application and tasks the daemon with scraping a url if both that url and daemon exist
- -stop url
        This stops the url from being scraped
- -list
        This lists all scraped urls to the terminal
- -clear
        This clears all files related to the daemon
- -kill
        This kills the daemon and then clears all files related to the daemon
- -print
        This prints out the scraped urls to output.txt
