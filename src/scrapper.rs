use chromiumoxide::{Page, Element};
use color_eyre::eyre::Result;
use url::Url;
 
use crate::{chrome::BROWSER, ARGS};

// Tworzenie nowej karty przeglądarki
pub async fn get_page (url: &Url) -> Result<Page> {
    Ok(BROWSER.clone().lock().await.as_ref().unwrap().browser.new_page(url.as_str()).await?)
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
pub async fn get_content (page: &Page) -> Result<Option<String>> {
    let args = ARGS.clone();

    // Jeżeli zapisujemy wyniki do CSV
    if args.csv.unwrap_or(false) {
        // Pobieramy zawartość strony
        let content = page.find_element("body").await?
            .inner_text().await?
            .unwrap_or_default()
            .replace("\n", " ");

        Ok(Some(content))
    } else {
        // Inaczej, zwracamy pustą zawartość, aby trzymać mniej danych
        Ok(None)
    }
}
 
