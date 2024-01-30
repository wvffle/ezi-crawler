use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use futures::StreamExt;
use derive_builder::Builder;

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use petgraph::dot::{Dot, Config};
use petgraph::graphmap::DiGraphMap;
use serde::Serialize;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use url::Url;

use par_dfs::r#async::{Bfs, Dfs, Node, NodeStream};

use crate::{scrapper, ARGS};

lazy_static! {
    static ref START_TIME: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    static ref NODE_CACHE: Arc<RwLock<HashMap<String, CrawlNode>>> = Arc::new(RwLock::new(HashMap::new()));
}

// Definicja strategii przeszukiwania
#[derive(Clone, Copy, Debug)]
pub enum CrawlStrategy {
    Bfs,
    Dfs
}

// Trait potrzebny do parsowania argumentów CLI
impl Into<clap::builder::OsStr> for CrawlStrategy {
    fn into(self) -> clap::builder::OsStr {
        match self {
            CrawlStrategy::Bfs => "bfs".into(),
            CrawlStrategy::Dfs => "dfs".into()
        }
    }
}

// Trait potrzebny do parsowania argumentów CLI
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


// Definicja węzła przeszukiwania
#[derive(Clone, Debug, Builder, Serialize)]
struct CrawlNode {
    link: String,

    #[serde(skip)]
    links: Vec<String>,

    title: Option<String>,
    description: Option<String>,
    content: Option<String>,
}

// Implementacja potrzebna do porównywania węzłów
// Chcemy, żeby dwa węzły były równe, jeśli mają taki sam link
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

// Tworzenie węzła na podstawie strony
async fn create_node(link: String) -> Result<CrawlNode> {
    let cached = NODE_CACHE.read().await.get(&link).cloned();
    if let Some(node) = cached {
        return Ok(node);
    }

    let url = Url::parse(link.as_str())?;
    let page = scrapper::get_page(&url).await?;
    let links = scrapper::get_links(&page).await?;
    let (title, description, content) = scrapper::get_content(&page).await?;

    let node = CrawlNodeBuilder::default()
        .link(link.clone())
        .links(links)
        .title(title)
        .description(description)
        .content(content)
        .build()?;

    NODE_CACHE.write().await.insert(link, node.clone());
    Ok(node)
}

// Implementacja traitu potrzebnego do asynchronicznego przeszukiwania
#[async_trait::async_trait]
impl Node for CrawlNode {
    type Error = color_eyre::Report;

    async fn children(
        self: Arc<Self>,
        _depth: usize
    ) -> Result<NodeStream<Self, Self::Error>> {
        // Sprawdzamy, czy nie przekroczyliśmy timeoutu
        let elapsed = { START_TIME.lock().unwrap().unwrap().elapsed() };
        if elapsed.as_secs() > ARGS.clone().timeout_secs.into() {
            // Jeśli tak, to zwracamy pustego streama
            return Ok(Box::pin(futures::stream::empty().boxed()))
        }

        // Tworzymy węzły dla każdej karty
        let links = self.links.clone();
        let nodes = links.into_iter()
            .map(|link| {
                tokio::task::spawn(create_node(link.to_string()))
            })
            .collect::<Vec<_>>();

        // Czekamy aż wszystkie węzły się utworzą
        let nodes = futures::future::join_all(nodes).await;

        // Zwracamy stream z utworzonymi węzłami
        let stream = futures::stream::iter(nodes)
            .map(|node| node.unwrap());

        Ok(Box::pin(stream.boxed()))
    }
}

// Funkcja do wypisywania drzewa
fn print_graph (link: String, nodes: &HashMap<String, CrawlNode>, max_depth: usize) {
    if ARGS.silent {
        return;
    }

    fn print_node (link: String, nodes: &HashMap<String, CrawlNode>, depth: usize, max_depth: usize, is_last: bool) {
        // Ładnie formatujemy wypisywane drzewo
        let x = if depth > 0 { "│ ".repeat(depth - 1) } else { "".to_string() };
        let y = if depth == 0 { "" } else if is_last { "╰╴" } else { "├╴" };
        println!("{}{}{}", x, y, link);

        // Jeśli przekroczyliśmy maksymalną głębokość, to kończymy
        // Zabezpiecza to przed nieskończoną pętlą
        if depth >= max_depth {
            return;
        }

        // Jeśli węzeł istnieje, to wypisujemy jego dzieci
        if let Some(node) = nodes.get(&link) {
            let last_link = node.links.last();
            for link in &node.links {
                print_node(link.to_string(), nodes, depth + 1, max_depth, last_link == Some(link));
            }
        }
    }

    // Wypisujemy korzeń
    print_node(link, nodes, 0, max_depth, false);
}

// Funkcja pomocnicza do mierzenia czasu
fn start_timer () {
    START_TIME.lock().unwrap().replace(Instant::now());
}

// Strategia przeszukiwania BFS
pub async fn bfs () -> Result<()> {
    let args = ARGS.clone();
    let url = args.url.clone();
    let max_depth = args.max_depth as usize;

    // Tworzymy kartę oraz węzeł dla korzenia
    let root = create_node(url.to_string()).await?;

    // Rozpoczynamy pomiar czasu dla timeoutu
    start_timer();

    // Uruchamiamy przeszukiwanie BFS
    let bfs = Bfs::<CrawlNode>::new(root.clone(), max_depth - 1, false);
    let out: Vec<Result<CrawlNode>> = bfs.collect().await;
    
    // Kolekcjonujemy wszystkie węzły w hashmapie link -> węzeł
    let mut map = HashMap::new();
    map.insert(root.link.clone(), root.clone());
    for node in out {
        let node = node?;
        map.insert(node.link.clone(), node);
    }

    // Wypisujemy drzewo
    print_graph(root.link.clone(), &map, max_depth);

    // Zapisujemy wyniki do pliku CSV
    save_csv(&map);

    // Zapisaujemy wyniki do pliku DOT
    save_graph(map).await?;

    Ok(())
}

// Strategia przeszukiwania DFS
pub async fn dfs () -> Result<()> {
    let args = ARGS.clone();
    let url = args.url.clone();
    let max_depth = args.max_depth as usize;

    // Tworzymy kartę oraz węzeł dla korzenia
    let root = create_node(url.to_string()).await?;

    // Rozpoczynamy pomiar czasu dla timeoutu
    start_timer();

    // Uruchamiamy przeszukiwanie DFS
    let dfs = Dfs::<CrawlNode>::new(root.clone(), max_depth - 1, false);
    let out: Vec<Result<CrawlNode>> = dfs.collect().await;

    // Kolekcjonujemy wszystkie węzły w hashmapie link -> węzeł
    let mut map = HashMap::new();
    map.insert(root.link.clone(), root.clone());
    for node in out {
        let node = node?;
        map.insert(node.link.clone(), node);
    }

    // Wypisujemy drzewo
    print_graph(root.link.clone(), &map, max_depth);

    // Zapisujemy wyniki do pliku CSV
    save_csv(&map);

    // Zapisaujemy wyniki do pliku DOT
    save_graph(map).await?;

    Ok(())
}

// Funkcja pomocnicza do zapisywania wyników do pliku CSV
fn save_csv (nodes: &HashMap<String, CrawlNode>) {
    let args = ARGS.clone();
    if args.csv {
        let mut wtr = csv::Writer::from_path("out.csv").unwrap();
        for node in nodes.values() {
            wtr.serialize(node).unwrap();
        }
        wtr.flush().unwrap();

        if !args.silent {
            eprintln!("Saved output to out.csv");
        }
    }
}

// Funkcja pomocnicza do zapisywania wyników do pliku DOT
async fn save_graph (nodes: HashMap<String, CrawlNode>) -> Result<()> {
    let args = ARGS.clone();
    if args.dot {

        // Rekonstrukcja grafu z hashmapy
        let mut graph = DiGraphMap::new();
        for node in nodes.values() {
            for link in &node.links {
                graph.add_edge(&node.link, link, ());
            }
        }

        // Zapis grafu do pliku DOT
        let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::GraphContentOnly]);
        let mut file = File::create("out.dot").await?;
        file.write_all(format!("digraph {{\nranksep = 8;\n{:?}\n}}", dot).as_bytes()).await?;

        if !args.silent {
            eprintln!("Saved output to out.dot");
        }
    }

    Ok(())
}
