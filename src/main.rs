use clap::{self, error::ErrorKind, Parser};
use hermis::spread_weight;
use rand::{self, seq::IteratorRandom, seq::SliceRandom, thread_rng};
use rand_pcg::Pcg64;
use rand_seeder;
use std::rc::Rc;
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

    let print_article = |article: &Article| {
        println!("---");
        println!("> {}", article.title());
        println!("> {}", article.description());
        println!("> {}", article.url());
    };

    let Ok(feeds) = config.parse_feeds() else {
        return Err(std::io::Error::other("foo"));
    };
    let seed = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let mut random: Pcg64 =
        rand_seeder::Seeder::from(config.seed.unwrap_or("skibbidytoilet".to_string())).make_rng();

    if args.lucky {
        feeds
            .into_iter()
            .map(|feed| feed.take(5))
            .flatten()
            .choose_multiple(&mut random, 10)
            .iter()
            .for_each(print_article);
    } else {
        let articles: Vec<Article> = feeds.into_iter().flatten().collect();
        spread_weight(articles.iter().collect())
            .into_iter()
            .choose_multiple(&mut random, config.page_size.unwrap_or(10))
            .into_iter()
            .map(|num| &articles[num])
            .for_each(print_article);
    }

    return Ok(());
}
