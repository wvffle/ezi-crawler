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

// Inicjalizacja parametrów CLI
lazy_static! {
    pub static ref ARGS: Arc<Args> = Arc::new(Args::parse());
}

// Parametry CLI
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    /// Strategia przeszukiwania
    #[arg(short, long, default_value = CrawlStrategy::Bfs)]
    strategy: CrawlStrategy,

    /// Adres URL do przeszukania
    #[arg(short, long)]
    url: Url,

    /// Maksymalna liczba wątków
    #[arg(short, long)]
    jobs: Option<u8>,

    /// Czy zapisywać wyniki do pliku CSV
    #[arg(short, long)]
    csv: Option<bool>,

    /// Timeout przeszukiwania w sekundach
    #[arg(short, long)]
    timeout_secs: Option<u16>,

    /// Maksymalna głębokość przeszukiwania
    #[arg(short = 'd', long)]
    max_depth: Option<u8>,

    // Generowanie wizualizacji grafu
    #[arg(short, long)]
    generate_visualization: Option<bool>,
}

 
fn main() -> Result<()> {
    color_eyre::install()?;

    let args = ARGS.clone();

    // Inicjalizacja asynchronicznego runtime (Tokio)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()

        // Ustawienie liczby
        .worker_threads(args.jobs.unwrap_or(1) as usize)
        .build()?;

    // Uruchomienie runtime
    rt.block_on(async {

        // Inicjalizacja przeglądarki
        create_browser().await.unwrap();

        // Uruchomienie crawlera w zależności od wybranej strategii
        match args.strategy {
            CrawlStrategy::Bfs => crawler::bfs().await.unwrap(),
            CrawlStrategy::Dfs => crawler::dfs().await.unwrap()
        }

        // Zamknięcie przeglądarki
        BROWSER.clone().lock().await.as_mut().unwrap().browser.close().await.unwrap()
    });
             
    Ok(())
}
 
