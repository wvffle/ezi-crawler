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
    /// Adres URL do przeszukania
    #[arg(short, long)]
    url: Url,

    /// Strategia przeszukiwania
    #[arg(short = 'S', long, default_value = CrawlStrategy::Bfs)]
    strategy: CrawlStrategy,

    /// Timeout przeszukiwania w sekundach
    #[arg(short, long)]
    timeout_secs: Option<u16>,

    /// Maksymalna głębokość przeszukiwania
    #[arg(short = 'd', long)]
    max_depth: Option<u8>,

    /// Czy zapisywać wyniki do pliku out.csv
    #[arg(short, long, default_value_t = false)]
    csv: bool,

    /// Czy zapisywać wizualizację grafu do pliku out.dot
    #[arg(short = 'D', long, default_value_t = false)]
    dot: bool,

    /// Własny User-Agent
    #[arg(short = 'U', long)]
    user_agent: Option<String>,

    /// Maksymalna liczba wątków
    #[arg(short, long)]
    jobs: Option<u8>,

    /// Czy uruchomić przeglądarkę w trybie headful
    #[arg(short = 'H', long, default_value_t = false)]
    headful: bool,

    /// Czy pobrać lokalną wersję Chromium
    #[arg(short, long, default_value_t = false)]
    fetch_chromium: bool,

    /// Czy wyświetlać logi
    #[arg(short, long, default_value_t = false)]
    silent: bool,
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
        let handle = create_browser().await.unwrap();

        // Uruchomienie crawlera w zależności od wybranej strategii
        match args.strategy {
            CrawlStrategy::Bfs => crawler::bfs().await.unwrap(),
            CrawlStrategy::Dfs => crawler::dfs().await.unwrap()
        }

        // Zamknięcie przeglądarki
        BROWSER.clone().lock().await.as_mut().unwrap().close().await.unwrap();
        handle.await.unwrap();
    });
             
    Ok(())
}
 
