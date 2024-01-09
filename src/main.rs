use std::sync::{Arc, Mutex};

use color_eyre::eyre::Result;
use url::Url;

mod chrome;
mod scrapper;
mod crawler;
 
 
#[tokio::main]
async fn main() -> Result<()> {
    let (browser, handle) = chrome::create_browser().await?;

    let browser = Arc::from(Mutex::from(browser));
    crawler::crawl(browser.clone(), Url::parse("https://funkwhale.audio")?, 6).await?;
 
    browser.lock().unwrap().close().await?;
    let _ = handle.await;
    Ok(())
}
 
