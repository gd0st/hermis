use clap::{self, Parser};
use rand::{self, seq::SliceRandom};
use rand_pcg::Pcg64;
use rand_seeder;
use std::fs::OpenOptions;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = ".config/hermis.json")]
    config: String,

    #[arg(short, long)]
    lucky: bool,
}

use hermis::{Article, Config};
fn main() -> anyhow::Result<()> {
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

    let feeds = config.parse_feeds()?;
    let mut rng: Pcg64 = rand_seeder::Seeder::from(config.seed()).make_rng();

    let articles: Vec<Article> = feeds
        .into_iter()
        .map(|feed| feed.take(5))
        .flatten()
        .collect();
    if true {
        articles
            .choose_multiple_weighted(&mut rng, config.limit(), |article| article.weight())?
            .for_each(|article| print_article(article));
    } else {
    }

    return Ok(());
}
