use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chromiumoxide::Page;
use futures::StreamExt;
use derive_builder::Builder;

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use serde::Serialize;
use url::Url;

use par_dfs::r#async::{Bfs, Dfs, Node, NodeStream};

use crate::{scrapper, ARGS};

lazy_static! {
    static ref START_TIME: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
}

#[derive(Clone, Copy, Debug)]
pub enum CrawlStrategy {
    Bfs,
    Dfs
}

impl Into<clap::builder::OsStr> for CrawlStrategy {
    fn into(self) -> clap::builder::OsStr {
        match self {
            CrawlStrategy::Bfs => "bfs".into(),
            CrawlStrategy::Dfs => "dfs".into()
        }
    }
}

impl FromStr for CrawlStrategy {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "bfs" => Ok(CrawlStrategy::Bfs),
            "dfs" => Ok(CrawlStrategy::Dfs),
            _ => Err(color_eyre::eyre::eyre!("Invalid strategy"))
        }
    }
}


#[derive(Clone, Debug, Builder, Serialize)]
struct CrawlNode {
    link: String,

    #[serde(skip)]
    depth: usize,

    #[serde(skip)]
    links: Vec<String>,

    content: Option<String>
}

impl Eq for CrawlNode {}

impl PartialEq for CrawlNode {
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}

impl Hash for CrawlNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.link.hash(state);
    }
}

async fn get_node(page: Page, depth: usize, link: String) -> Result<CrawlNode> {
    let links = scrapper::get_links(&page).await?;
    let content = scrapper::get_content(&page).await?;

    let node = CrawlNodeBuilder::default()
        .link(link)
        .links(links)
        .content(content)
        .depth(depth)
        .build()?;

    Ok(node)
}

#[async_trait::async_trait]
impl Node for CrawlNode {
    type Error = color_eyre::Report;

    async fn children(
        self: Arc<Self>,
        depth: usize
    ) -> Result<NodeStream<Self, Self::Error>> {
        let elapsed = { START_TIME.lock().unwrap().unwrap().elapsed() };
        if elapsed.as_secs() > ARGS.clone().timeout_secs.unwrap_or(u16::MAX).into() {
            return Ok(Box::pin(futures::stream::empty().boxed()))
        }

        let depth = depth + 1;
        let links = self.links.clone();

        let pages = links.into_iter()
            .map(|link| async {
                let url = Url::parse(link.as_str()).unwrap();
                (link, scrapper::get_page(&url).await.unwrap())
            })
            .collect::<Vec<_>>();

        let pages = futures::future::join_all(pages).await;
        let nodes = pages.into_iter()
            .map(|(link, page)| {
                tokio::task::spawn(get_node(page, depth, link.to_string()))
            })
            .collect::<Vec<_>>();

        let nodes = futures::future::join_all(nodes).await;
        let stream = futures::stream::iter(nodes)
            .map(|node| node.unwrap());

        Ok(Box::pin(stream.boxed()))
    }
}

fn print_graph (link: String, nodes: &HashMap<String, CrawlNode>, max_depth: usize) {
    fn print_node (link: String, nodes: &HashMap<String, CrawlNode>, depth: usize, max_depth: usize, is_last: bool) {
        let x = if depth > 0 { "│ ".repeat(depth - 1) } else { "".to_string() };
        let y = if depth == 0 { "" } else if is_last { "╰╴" } else { "├╴" };
        println!("{}{}{}", x, y, link);

        if depth >= max_depth {
            return;
        }

        if let Some(node) = nodes.get(&link) {
            let last_link = node.links.last();
            for link in &node.links {
                print_node(link.to_string(), nodes, depth + 1, max_depth, last_link == Some(link));
            }
        }
    }

    print_node(link, nodes, 0, max_depth, false);
}

fn start_timer () {
    START_TIME.lock().unwrap().replace(Instant::now());
}

pub async fn bfs () -> Result<()> {
    let args = ARGS.clone();
    let url = args.url.clone();
    let max_depth: usize = args.max_depth.unwrap_or(u8::MAX).into();

    let page = scrapper::get_page(&url).await.unwrap();
    let root = get_node(page, 1, url.to_string()).await?;

    start_timer();
    let bfs = Bfs::<CrawlNode>::new(root.clone(), max_depth - 1, false);
    let out: Vec<Result<CrawlNode>> = bfs.collect().await;
    
    let mut map = HashMap::new();
    map.insert(root.link.clone(), root.clone());
    for node in out {
        let node = node?;
        map.insert(node.link.clone(), node);
    }

    print_graph(root.link.clone(), &map, max_depth);
    save_csv(&map);
    Ok(())
}

pub async fn dfs () -> Result<()> {
    let args = ARGS.clone();
    let url = args.url.clone();
    let max_depth: usize = args.max_depth.unwrap_or(u8::MAX).into();

    let page = scrapper::get_page(&url).await.unwrap();
    let root = get_node(page, 1, url.to_string()).await?;

    start_timer();
    let dfs = Dfs::<CrawlNode>::new(root.clone(), max_depth - 1, false);
    let out: Vec<Result<CrawlNode>> = dfs.collect().await;

    let mut map = HashMap::new();
    map.insert(root.link.clone(), root.clone());
    for node in out {
        let node = node?;
        map.insert(node.link.clone(), node);
    }

    print_graph(root.link.clone(), &map, max_depth);
    save_csv(&map);
    Ok(())
}

fn save_csv (nodes: &HashMap<String, CrawlNode>) {
    let args = ARGS.clone();
    if args.csv.unwrap_or(false) {
        let mut wtr = csv::Writer::from_path("out.csv").unwrap();
        for node in nodes.values() {
            wtr.serialize(node).unwrap();
        }
        wtr.flush().unwrap();
        eprintln!("Saved output to out.csv");
    }
}
