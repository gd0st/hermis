use clap::{self, error::ErrorKind, Parser};
use rand::{self, seq::IteratorRandom, thread_rng};
use std::{fs::OpenOptions, io};

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = ".config/hermis.json")]
    config: String,

    #[arg(short, long)]
    lucky: bool,
}

use hermis::{Article, Config, Feed};
fn main() -> io::Result<()> {
    let args = Args::parse();
    let home = std::env::var("HOME").unwrap_or_default().to_string();
    let sources: Vec<String> = vec![
        args.config.to_string(),
        format!("{home}/.config/hermis.json"),
    ];

    let reader = OpenOptions::new().read(true).open(args.config)?;
    let config: Config = serde_json::from_reader(reader)?;

    let print_article = |article: Article| {
        println!("---");
        println!("> {}", article.title());
        println!("> {}", article.description());
        println!("> {}", article.url());
    };

    if let Ok(feeds) = config.parse_feeds() {
        if args.lucky {
            let mut random = rand::thread_rng();

            feeds
                .into_iter()
                .map(|feed| feed.take(5))
                .flatten()
                .choose_multiple(&mut random, 10)
                .into_iter()
                .for_each(print_article);
        } else {
            feeds
                .into_iter()
                .map(|feed| feed.take(5))
                .flatten()
                .for_each(print_article)
        }
    } else {
        return Err(std::io::Error::other("foo"));
    };

    return Ok(());

    let Some(config) = Config::scan_paths(sources.iter().map(|s| s.as_str()).collect()) else {
        return Err(io::Error::other("no config"));
    };

    println!("{:?}", &config);

    Ok(())
}
