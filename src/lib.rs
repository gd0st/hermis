use anyhow;
use reqwest::blocking::{self, Client, ClientBuilder};
use rss::{self, Channel, ChannelBuilder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use std::{fs::OpenOptions, io};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    sources: Vec<String>,
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
    pub fn parse_feeds(&self) -> anyhow::Result<Vec<Feed>> {
        let client = Client::default();

        let mut feeds = vec![];

        for source in self.sources.iter() {
            let res = client.get(source).send()?.text()?;
            let reader = BufReader::new(res.as_bytes());
            let channel = Channel::read_from(reader)?;
            let articles: Vec<Article> = channel.items().iter().map(Article::from).collect();
            feeds.push(Feed {
                name: channel.title().to_string(),
                articles,
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

pub struct Article {
    title: String,
    description: String,
    url: String,
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
}

impl From<&rss::Item> for Article {
    fn from(value: &rss::Item) -> Self {
        Article {
            title: value.title().unwrap_or("default").to_string(),
            description: value.description().unwrap_or("default").to_string(),
            url: value.link().unwrap_or("https://example.org").to_string(),
        }
    }
}

pub struct Feed {
    name: String,
    articles: Vec<Article>,
}

impl Feed {
    pub fn articles(&self) -> Vec<&Article> {
        self.articles.iter().collect()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<Config> for Vec<Feed> {
    type Error = anyhow::Error;

    fn try_from(value: Config) -> anyhow::Result<Self> {
        let client = Client::default();

        for source in value.sources {
            if let Ok(blob) = client.get(source).send() {
                let buff = BufReader::new(blob.text()?.as_bytes());
            }
        }

        todo!()
    }
}
