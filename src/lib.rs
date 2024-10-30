use anyhow;
use reqwest::blocking::Client;
use rss::{self, Channel};
use serde::{Deserialize, Serialize};
use serde_json;

use std::io::BufReader;
use std::path::PathBuf;

use std::{fs::OpenOptions, io};

const DEFAULT_SEED: &str = "skibbidytoilet";

#[derive(Serialize, Deserialize, Debug)]
struct Source {
    link: String,
    weight: Option<isize>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug, Clone, Default)]
pub struct ArticleBuilder {
    title: String,
    description: String,
    url: String,
    weight: i32,
}

// TODO feels somewhat useless
impl ArticleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    pub fn weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }

    pub fn build(self) -> Article {
        Article {
            title: self.title,
            description: self.description,
            url: self.url,
            weight: self.weight,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Article {
    title: String,
    description: String,
    url: String,
    weight: i32,
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Feed {
    name: String,
    articles: Vec<Article>,
    weight: i32,
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
        }
    }
}

impl Feed {
    pub fn articles(&self) -> Vec<&Article> {
        self.articles.iter().collect()
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
