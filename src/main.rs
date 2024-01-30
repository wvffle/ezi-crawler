use chrome::{create_browser, BROWSER};
use color_eyre::eyre::Result;
use crawler::CrawlStrategy;
use url::Url;
use clap::Parser;
use std::sync::Arc;
use lazy_static::lazy_static;

mod chrome;
mod scrapper;
mod crawler;

lazy_static! {
    pub static ref ARGS: Arc<Args> = Arc::new(Args::parse());
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = CrawlStrategy::Bfs)]
    strategy: CrawlStrategy,

    #[arg(short, long)]
    url: Url,

    #[arg(short, long)]
    jobs: Option<u8>,

    #[arg(short, long)]
    csv: Option<bool>,

    #[arg(short, long)]
    timeout_secs: Option<u16>,

    #[arg(short = 'd', long)]
    max_depth: Option<u8>,

    #[arg(short, long)]
    generate_sitemap: bool,
}

 
fn main() -> Result<()> {
    color_eyre::install()?;

    // let args = Args::parse();
    // ARGS.write().unwrap().replace(args.clone());
    let args = ARGS.clone();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(args.jobs.unwrap_or(1) as usize)
        .build()?;

    rt.block_on(async {
        create_browser().await.unwrap();

        match args.strategy {
            CrawlStrategy::Bfs => crawler::bfs().await.unwrap(),
            CrawlStrategy::Dfs => crawler::dfs().await.unwrap()
        }

        BROWSER.clone().lock().await.as_mut().unwrap().browser.close().await.unwrap()
    });
             
    Ok(())
}
 
