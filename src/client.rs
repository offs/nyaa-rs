use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client as HttpClient;
use scraper::{ElementRef, Html, Selector};

use crate::model::{Category, Sort, Torrent};

static ITEM_SELECTOR: OnceLock<Selector> = OnceLock::new();
static TITLE_SELECTOR: OnceLock<Selector> = OnceLock::new();
static LINK_SELECTOR: OnceLock<Selector> = OnceLock::new();
static MAGNET_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SIZE_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DATE_SELECTOR: OnceLock<Selector> = OnceLock::new();
static SEEDERS_SELECTOR: OnceLock<Selector> = OnceLock::new();
static LEECHERS_SELECTOR: OnceLock<Selector> = OnceLock::new();
static DOWNLOADS_SELECTOR: OnceLock<Selector> = OnceLock::new();

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const BASE_URL: &str = "https://nyaa.si";

#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            http: HttpClient::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .unwrap_or_default(),
        }
    }
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn search(
        &self,
        query: &str,
        category: Category,
        sort: Sort,
        page: u32,
    ) -> Result<Vec<Torrent>> {
        let encoded_query = urlencoding::encode(query);
        let url =
            format!("{BASE_URL}/?f=0&c={category}&q={encoded_query}&s={sort}&o=desc&p={page}");
        let response = self.http.get(&url).send().await?.text().await?;

        tokio::task::spawn_blocking(move || extract(&response)).await?
    }
}

fn extract(html: &str) -> Result<Vec<Torrent>> {
    let document = Html::parse_document(html);
    let selector = ITEM_SELECTOR.get_or_init(|| Selector::parse("table>tbody>tr").unwrap());

    document
        .select(selector)
        .filter_map(|item| extract_torrent(item).ok())
        .collect::<Vec<_>>()
        .pipe(Ok)
}

impl<T> Pipe for T {}

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

fn extract_torrent(item: ElementRef) -> Result<Torrent> {
    let title = extract_text(
        item,
        TITLE_SELECTOR
            .get_or_init(|| Selector::parse("td:nth-of-type(2)>a:not(.comments)").unwrap()),
    )?;

    let link_sel =
        LINK_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(3)>a:first-child").unwrap());
    let link_path = item
        .select(link_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .context("link not found")?;
    let link = format!("{BASE_URL}{link_path}");

    let magnet_sel = MAGNET_SELECTOR
        .get_or_init(|| Selector::parse("td:nth-of-type(3)>a:nth-child(2)").unwrap());
    let magnet_url = item
        .select(magnet_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .unwrap_or("")
        .to_string();

    let size = extract_text(
        item,
        SIZE_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(4)").unwrap()),
    )?;

    let mut date = extract_text(
        item,
        DATE_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(5)").unwrap()),
    )?;

    if date.len() > 10 {
        date.truncate(10);
    }

    let seeders = extract_u32(
        item,
        SEEDERS_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(6)").unwrap()),
    );
    let leechers = extract_u32(
        item,
        LEECHERS_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(7)").unwrap()),
    );
    let downloads = extract_u32(
        item,
        DOWNLOADS_SELECTOR.get_or_init(|| Selector::parse("td:nth-of-type(8)").unwrap()),
    );

    Ok(Torrent {
        title,
        link,
        magnet_url,
        date,
        seeders,
        leechers,
        downloads,
        size,
    })
}

fn extract_text(item: ElementRef, selector: &Selector) -> Result<String> {
    Ok(item
        .select(selector)
        .next()
        .context("element not found")?
        .text()
        .collect::<String>()
        .trim()
        .to_string())
}

fn extract_u32(item: ElementRef, selector: &Selector) -> u32 {
    extract_text(item, selector)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
