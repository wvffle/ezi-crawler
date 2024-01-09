use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::{Arc, Mutex};

use color_eyre::eyre::Result;
use url::Url;

 
use chromiumoxide::browser::Browser;

use crate::scrapper;

#[derive(Clone)]
struct CrawlNode {
    parent_url: Option<String>,
    url: String,
    depth: usize,
    links: Vec<String>
}


pub async fn crawl (browser: Arc<Mutex<Browser>>, url: Url, max_depth: usize) -> Result<()> {
    let mut visited = HashMap::new();
    let mut queue = VecDeque::<CrawlNode>::new();



    let page = { browser.lock().unwrap().new_page(url.as_str()).await? };
    let links = scrapper::get_links(&page, url.as_str()).await?;
    let root = CrawlNode {
        parent_url: None,
        url: url.to_string(),
        depth: 1,
        links
    };

    queue.push_back(root);

    eprintln!("[ ] {:indent$}{}", "", url, indent = 0);
    while let Some(node) = queue.pop_front() {
        if node.depth > max_depth { continue; }

        for link in &node.links {
            if visited.contains_key(link) {
                eprintln!("[V] {:indent$}{}", "", node.url, indent = node.depth * 2);
                continue;
            }
            
            visited.insert(link.clone(), node.clone());
            eprintln!("[ ] {:indent$}{}", "", link, indent = node.depth * 2);

            let page = { browser.lock().unwrap().new_page(link.as_str()).await? };
            let links = scrapper::get_links(&page, link.as_str()).await?;

            queue.push_back(CrawlNode {
                parent_url: Some(node.url.clone()),
                url: link.clone(),
                depth: node.depth + 1,
                links
            });
        }

    }

    Ok(())
}
