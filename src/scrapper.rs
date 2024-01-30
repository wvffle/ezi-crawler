use chromiumoxide::{Page, Element};
use color_eyre::eyre::Result;
use url::Url;
 
use crate::{chrome::BROWSER, ARGS};

// Tworzenie nowej karty przeglądarki
pub async fn get_page (url: &Url) -> Result<Page> {
    Ok(BROWSER.clone().lock().await.as_ref().unwrap().new_page(url.as_str()).await?)
}

// Pobieranie linków z karty przeglądarki
pub async fn get_links (page: &Page) -> Result<Vec<String>> {
    // Znajdowanie wszystkich elementów <a> na stronie
    let anchors: Vec<Element> = page.find_elements("a[href]").await
        // Jeśli wystąpi błąd, to zwracamy pustą listę
        .or_else(|_| Ok::<Vec<Element>, color_eyre::eyre::ErrReport>(Vec::new()))
        .unwrap();
 
    let mut links = Vec::with_capacity(anchors.len());
    let url: Url = Url::parse(&page.url().await?.unwrap())?;

    // Filtrujemy linki, które nie są http lub https, albo są błędne
    for link in anchors {
        let href = link.attribute("href").await?;
        if let Some(mut address) = href {
            if address.starts_with("//") {
                address = "https:".to_string() + address.as_str();
            }

            // Łączymy link z adresem karty
            // Ta operacja pozwala nam na połączenie poprzedniego adresu z linkiem `/asdf` lub `asdf` dobierając odpowiednią strategię
            // Jeżeli link zaczyna się od protokołu, adres karty będzie nadpisany
            let mut link = url.clone().join(&address)?;
            if link.scheme() != "http" && link.scheme() != "https" {
                continue;
            }

            // Ustawiamy fragment (hash) na `None` aby uniknąć duplikatów
            link.set_fragment(None);

            // Dodajemy link do przefiltrowanej listy
            links.push(link.to_string());
        }
    }
 
    Ok(links)
}

// Pobieranie zawartości strony
pub async fn get_content (page: &Page) -> Result<(Option<String>, Option<String>, Option<String>)> {
    let args = ARGS.clone();

    // Jeżeli zapisujemy wyniki do CSV
    if args.csv {
        // Pobieramy tytuł strony
        let mut title = match page.find_element("title").await {
            Ok(title) => Some(title.inner_text().await?.unwrap_or_default().replace("\n", " ")),
            _ => None
        };

        // Jeżeli nie ma tytułu, to pobieramy og:title (edge case, gdy studenci zamiast w HTML umieją w socjale)
        if title.is_none() {
            title = match page.find_element("meta[property=og:title]").await {
            Ok(title) => Some(title.attribute("content").await?.unwrap_or_default().replace("\n", " ")),
            _ => None
            };
        }

        // Pobieramy opis strony
        let mut description = match page.find_element("meta[name=description]").await {
            Ok(description) => Some(description.attribute("content").await?.unwrap_or_default().replace("\n", " ")),
            _ => None
        };

        // Jeżeli nie ma opisu, to pobieramy og:description
        if description.is_none() {
            description = match page.find_element("meta[property=og:description]").await {
                Ok(description) => Some(description.attribute("content").await?.unwrap_or_default().replace("\n", " ")),
                _ => None
            };
        }

        // Pobieramy zawartość strony
        let content = match page.find_element("body").await {
            Ok(content) => Some(content.inner_text().await?.unwrap_or_default().replace("\n", " ")),
            _ => None
        };

        Ok((title, description, content))
    } else {
        // Inaczej, zwracamy pustą zawartość, aby trzymać mniej danych
        Ok((None, None, None))
    }
}
 
