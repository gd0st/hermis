use clap::{self, error::ErrorKind, Parser};
use std::{fs::OpenOptions, io};

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = ".config/hermis.json")]
    config: String,
}

use hermis::{Config, Feed};
fn main() -> io::Result<()> {
    let args = Args::parse();
    let home = std::env::var("HOME").unwrap_or_default().to_string();
    let sources: Vec<String> = vec![
        args.config.to_string(),
        format!("{home}/.config/hermis.json"),
    ];

    let reader = OpenOptions::new().read(true).open(args.config)?;
    let config: Config = serde_json::from_reader(reader)?;

    if let Ok(feeds) = config.parse_feeds() {
        for feed in feeds {
            println!("+ {}", feed.name());
            for article in feed.articles().iter().take(5) {
                println!("---");
                println!("> {}", article.title());
                println!("> {}", article.description());
                println!("> {}", article.url());
            }
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
