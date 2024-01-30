use chromiumoxide::{Page, Element};
use color_eyre::eyre::Result;
use url::Url;
 
use crate::{chrome::BROWSER, ARGS};

pub async fn get_page (url: &Url) -> Result<Page> {
    Ok(BROWSER.clone().lock().await.as_ref().unwrap().browser.new_page(url.as_str()).await?)
}

pub async fn get_links (page: &Page) -> Result<Vec<String>> {
    let anchors: Vec<Element> = page.find_elements("a[href]").await
        .or_else(|_| Ok::<Vec<Element>, color_eyre::eyre::ErrReport>(Vec::new()))
        .unwrap();
 
    let mut links = Vec::with_capacity(anchors.len());
    let url: Url = Url::parse(&page.url().await?.unwrap())?;
    for link in anchors {
        let href = link.attribute("href").await?;
        if let Some(mut address) = href {
            if address.starts_with("//") {
                address = "https:".to_string() + address.as_str();
            }

            let mut link = url.clone().join(&address)?;
            if link.scheme() != "http" && link.scheme() != "https" {
                continue;
            }

            link.set_fragment(None);
            links.push(link.to_string());
        }
    }
 
    Ok(links)
}

pub async fn get_content (page: &Page) -> Result<Option<String>> {
    let args = ARGS.clone();
    if args.csv.unwrap_or(false) {
        let content = page.find_element("body").await?
            .inner_text().await?
            .unwrap_or_default()
            .replace("\n", " ");

        Ok(Some(content))
    } else {
        Ok(None)
    }
}
 
