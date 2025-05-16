use anyhow;
pub mod web;
use rand::seq::SliceRandom;
use rand::{rngs, Rng};
use rand_pcg::Pcg64;
use reqwest::blocking::Client;
use rss::{self, Channel};
use serde::{Deserialize, Serialize};
use serde_json;

use std::io::BufReader;
use std::path::PathBuf;

use std::{fs::OpenOptions, io};

const DEFAULT_SEED: &str = "skibbidytoilet";

pub fn seeded_rng(seed: &str) -> impl Rng {
    let rng: Pcg64 = rand_seeder::Seeder::from(seed).make_rng();
    rng
}

pub fn weighted_collection<'a>(
    feeds: Vec<&'a Feed>,
    count: usize,
    rng: &mut impl Rng,
) -> anyhow::Result<Vec<&'a Article>> {
    // holy hell
    let articles = feeds
        .iter()
        .map(|feed| feed.articles(true))
        .flatten()
        .collect::<Vec<&Article>>()
        .choose_multiple_weighted(rng, count, |article| article.weight())?
        .into_iter()
        .map(|article| *article)
        .collect();
    Ok(articles)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    link: String,
    weight: Option<isize>,
    skim: Option<usize>, // How many stories to take off the top
}

pub async fn parse_sources(sources: Vec<&Source>) -> anyhow::Result<Vec<Feed>> {
    let client = reqwest::Client::default();
    let mut feeds = vec![];

    for source in sources.into_iter() {
        let weight = source.weight.unwrap_or(1);
        let res = client.get(&source.link).send().await?.text().await?;
        let reader = BufReader::new(res.as_bytes());
        let channel = Channel::read_from(reader)?;

        let articles: Vec<Article> = channel
            .items()
            .iter()
            .map(|item| {
                let mut article = Article::from(item).publisher(channel.title());

                article.weight = weight as i32;
                article
            })
            .collect();

        feeds.push(Feed {
            name: channel.title().to_string(),
            articles,
            weight: weight as i32,
            skim: source.skim.unwrap_or(20),
        })
    }
    Ok(feeds)
}

impl Source {
    pub fn new(link: String) -> Self {
        Self {
            link,
            weight: None,
            skim: None,
        }
    }

    pub fn weight(mut self, weight: isize) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn skim(mut self, skim: usize) -> Self {
        self.skim = Some(skim);
        self
    }
}

impl TryFrom<Source> for Feed {
    type Error = anyhow::Error;
    fn try_from(source: Source) -> Result<Self, Self::Error> {
        let config = Config {
            sources: vec![source],
            page_size: None,
            seed: None,
            limit: None,
        };

        let mut feed = config.parse_feeds()?;
        Ok(feed.pop().unwrap())
    }
}

impl<'a> From<&'a str> for Source {
    fn from(link: &'a str) -> Self {
        Self {
            link: link.to_string(),
            weight: None,
            skim: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
///! todo!()
/// Allow weights to sources, favorite authors etc
pub struct Config {
    sources: Vec<Source>,
    page_size: Option<usize>,
    seed: Option<String>,
    limit: Option<usize>,
}

impl Config {
    pub fn scan_paths<'a>(paths: Vec<&'a str>) -> Option<Config> {
        for path in paths {
            let path = PathBuf::from(path);
            let Ok(reader) = OpenOptions::new().read(true).open(path) else {
                continue;
            };
            let Ok(config) = serde_json::from_reader(reader) else {
                continue;
            };
            config
        }
        None
    }

    pub fn append_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(10)
    }

    pub fn seed(&self) -> &str {
        if let Some(seed) = &self.seed {
            seed.as_str()
        } else {
            DEFAULT_SEED
        }
    }

    pub async fn async_parse_fetch(&self) -> anyhow::Result<Vec<Feed>> {
        let client = reqwest::Client::default();
        let mut feeds = vec![];
        for source in self.sources.iter() {
            let weight = match source.weight {
                Some(weight) => weight,
                None => 1,
            };

            let res = client.get(&source.link).send().await?.text().await?;
            let reader = BufReader::new(res.as_bytes());
            let channel = Channel::read_from(reader)?;
            let articles: Vec<Article> = channel
                .items()
                .iter()
                .map(|item| {
                    let mut article = Article::from(item).publisher(channel.title());

                    article.weight = weight as i32;
                    article
                })
                .collect();

            feeds.push(Feed {
                name: channel.title().to_string(),
                articles,
                weight: weight as i32,
                skim: source.skim.unwrap_or(20),
            })
        }
        Ok(feeds)
    }
    pub fn parse_feeds(&self) -> anyhow::Result<Vec<Feed>> {
        let client = Client::default();

        let mut feeds = vec![];

        for source in self.sources.iter() {
            let weight = match source.weight {
                Some(weight) => weight,
                None => 1,
            };
            let res = client.get(&source.link).send()?.text()?;
            let reader = BufReader::new(res.as_bytes());
            let channel = Channel::read_from(reader)?;
            let articles: Vec<Article> = channel
                .items()
                .iter()
                .map(|item| {
                    let mut article = Article::from(item);
                    article.weight = weight as i32;
                    article
                })
                .collect();

            feeds.push(Feed {
                name: channel.title().to_string(),
                articles,
                weight: weight as i32,
                skim: source.skim.unwrap_or(20),
            })
        }
        Ok(feeds)
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = std::io::Error;
    fn try_from(value: PathBuf) -> io::Result<Self> {
        let reader = OpenOptions::new().read(true).open(value)?;
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Article {
    title: String,
    description: String,
    url: String,
    weight: i32,
    publisher: Option<String>,
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn publisher(mut self, publisher: &str) -> Self {
        self.publisher = Some(publisher.to_string());
        self
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }
}

impl From<&rss::Item> for Article {
    fn from(value: &rss::Item) -> Self {
        Article {
            title: value.title().unwrap_or("default").to_string(),
            description: value.description().unwrap_or("default").to_string(),
            url: value.link().unwrap_or("https://example.org").to_string(),
            weight: 0,
            publisher: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Feed {
    name: String,
    articles: Vec<Article>,
    weight: i32,
    skim: usize, // how many articles to take
}

impl PartialEq for Feed {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

use std::cmp::Ordering;
impl PartialOrd for Feed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.weight == other.weight {
            return Some(Ordering::Equal);
        }

        if self.weight > other.weight {
            return Some(Ordering::Greater);
        }

        if self.weight < other.weight {
            return Some(Ordering::Less);
        }
        None
    }
    fn ge(&self, other: &Self) -> bool {
        self.weight >= other.weight
    }

    fn gt(&self, other: &Self) -> bool {
        self.weight > other.weight
    }

    fn le(&self, other: &Self) -> bool {
        self.weight <= other.weight
    }

    fn lt(&self, other: &Self) -> bool {
        self.weight < other.weight
    }
}

impl Default for Feed {
    fn default() -> Self {
        Feed {
            name: "".to_string(),
            articles: vec![],
            weight: 1,
            skim: 5,
        }
    }
}

impl Feed {
    pub fn articles(&self, with_skim: bool) -> Vec<&Article> {
        if with_skim {
            self.articles.iter().take(self.skim).collect()
        } else {
            self.articles.iter().collect()
        }
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }

    pub fn set_weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Iterator for Feed {
    type Item = Article;

    fn next(&mut self) -> Option<Self::Item> {
        self.articles.pop()
    }
}

impl TryFrom<Config> for Vec<Feed> {
    type Error = anyhow::Error;

    fn try_from(value: Config) -> anyhow::Result<Self> {
        let client = Client::default();

        for source in value.sources {
            if let Ok(blob) = client.get(&source.link).send() {
                let buff = BufReader::new(blob.text()?.as_bytes());
            }
        }

        todo!()
    }
}
