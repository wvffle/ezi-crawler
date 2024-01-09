use color_eyre::eyre::Result;
use url::Url;
 
use chromiumoxide::page::Page;

pub async fn get_links (page: &Page, url: &str) -> Result<Vec<String>> {
    let anchors = page.find_elements("a[href]")
        .await?;
 
    let mut links = Vec::with_capacity(anchors.len());
    let url = Url::parse(url)?;
    for link in anchors {
        let href = link.attribute("href").await?;
        if let Some(mut address) = href {
            if address.starts_with("//") {
                address = "https:".to_string() + address.as_str();
            }

            if !address.starts_with("http") && !address.starts_with("/") {
                continue;
            }

            links.push(url.clone().join(&address)?.to_string());
        }
    }
 
    Ok(links)
}

