use std::sync::Arc;

use color_eyre::{eyre::eyre, Result};
 
use futures::StreamExt;
pub(crate) use chromiumoxide::browser::{Browser, BrowserConfig};
use lazy_static::lazy_static;
use tokio::sync::Mutex;

// Tworzymy statyczne odniesienie do przeglądarki
lazy_static! {
    pub static ref BROWSER: Arc<Mutex<Option<BrowserConnection>>> = Arc::from(Mutex::new(None));
}


// Struktura przechowująca połączenie z przeglądarką
pub struct BrowserConnection {
    handle: tokio::task::JoinHandle<()>,
    pub browser: Browser
}
 
// Uruchamianie przeglądarki
pub async fn create_browser() -> Result<()> {
    let config = BrowserConfig::builder()
        .disable_default_args()
        .args(CHROME_ARGS)
        // NOTE: Odpalamy w trybie headfull
        .with_head()
        .enable_cache()
        .request_timeout(std::time::Duration::from_secs(30))
        .build().map_err(|e| eyre!(e))?;
 
    let (browser, mut handler) = Browser::launch(config).await?;
 
    // Tworzymy nowy task, który cały czas będzie obsługiwał przeglądarkę
    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    // Nadpisujemy statyczne odniesienie do przeglądarki
    BROWSER.lock().await.replace(BrowserConnection {
        handle,
        browser
    });
     
    Ok(())
}

// Argumenty przeglądarki
// https://github.com/a11ywatch/chrome/blob/main/src/main.rs#L13
static CHROME_ARGS: [&'static str; 58] = [
    //  NOTE:Odpalamy w trybie headfull
    // "--headless",
    "--no-sandbox",
    "--no-first-run",
    "--hide-scrollbars",
    // "--allow-pre-commit-input",
    // "--user-data-dir=~/.config/google-chrome",
    "--allow-running-insecure-content",
    "--autoplay-policy=user-gesture-required",
    "--ignore-certificate-errors",
    "--no-default-browser-check",
    "--no-zygote",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage", // required or else docker containers may crash not enough memory
    "--disable-threaded-scrolling",
    "--disable-demo-mode",
    "--disable-dinosaur-easter-egg",
    "--disable-fetching-hints-at-navigation-start",
    "--disable-site-isolation-trials",
    "--disable-web-security",
    "--disable-threaded-animation",
    "--disable-sync",
    "--disable-print-preview",
    "--disable-partial-raster",
    "--disable-in-process-stack-traces",
    "--disable-v8-idle-tasks",
    "--disable-low-res-tiling",
    "--disable-speech-api",
    "--disable-smooth-scrolling",
    "--disable-default-apps",
    "--disable-prompt-on-repost",
    "--disable-domain-reliability",
    "--disable-component-update",
    "--disable-background-timer-throttling",
    "--disable-breakpad",
    "--disable-software-rasterizer",
    "--disable-extensions",
    "--disable-popup-blocking",
    "--disable-hang-monitor",
    "--disable-image-animation-resync",
    "--disable-client-side-phishing-detection",
    "--disable-component-extensions-with-background-pages",
    "--disable-ipc-flooding-protection",
    "--disable-background-networking",
    "--disable-renderer-backgrounding",
    "--disable-field-trial-config",
    "--disable-back-forward-cache",
    "--disable-backgrounding-occluded-windows",
    // "--enable-automation",
    "--log-level=3",
    "--enable-logging=stderr",
    "--enable-features=SharedArrayBuffer,NetworkService,NetworkServiceInProcess",
    "--metrics-recording-only",
    "--use-mock-keychain",
    "--force-color-profile=srgb",
    "--mute-audio",
    "--no-service-autorun",
    "--password-store=basic",
    "--export-tagged-pdf",
    "--no-pings",
    "--use-gl=swiftshader",
    "--window-size=1920,1080",
    "--disable-features=AudioServiceOutOfProcess,IsolateOrigins,site-per-process,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DestroyProfileOnBrowserClose,MediaRouter,DialMediaRouteProvider,AcceptCHFrame,AutoExpandDetailsElement,CertificateTransparencyComponentUpdater,AvoidUnnecessaryBeforeUnloadCheckSync,Translate"
];


