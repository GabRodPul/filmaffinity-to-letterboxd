use core::time;
use std::{error::Error, fs, time::Instant};
use itertools::Itertools;
use select::{
    document::Document,
    predicate::{Attr, Name}
};

use scraper::{Html, Selector};
use select::predicate::Class;
use tokio::main;

use clap::{Arg, Parser};

/// Arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ID of the user to scrape
    #[arg(short, long)]
    user_id: u32,

    /// Number of pages to scrape
    #[arg(short, long)]
    page_count: usize,

    #[arg(short, long)]
    chromedriver_port: u16
}

#[tokio::main]
async fn main() {
    scrape().await
}

async fn scrape() {
    // let args = Args::parse();
    let args = Args::parse();
    let base_url: &str = &format!("https://www.filmaffinity.com/en/userratings.php?user_id={}&orderby=4", args.user_id);
    let chromedriver_url: &str = &format!("http://localhost:{}", args.chromedriver_port);
    let mut data = Vec::<String>::with_capacity(50 * args.page_count);
    data.push("Title,Year,Directors,Rating10".to_owned());

    for p in 1..=args.page_count {
        let url = format!("{}&p={}&chv=list", base_url, p);
        println!("{}", url);

        // FilmAffinity needs JS to do it's thing in order to display content
        // Plain HTML requests won't work
        use fantoccini::{ClientBuilder};
        let c = ClientBuilder::native().connect(chromedriver_url).await.unwrap();
        c.goto(&url).await.unwrap();
        // c.maximize_window().await.unwrap();
        // c.execute("document.getElementById('disagree-btn').click()", vec![]).await.unwrap();
        // println!("{}", c.title().await.unwrap());

        // let text: String = match reqwest::get(format!("{}&p={}&chv=list", BASE_URL, p)).await {
        let text: String = match c.source().await {
            Err(e) => {
                println!("{}", e);
                return;
            },
            Ok(r) => r
        };
        // println!("{}", text);
        let d = Document::from(&text[..]);
        drop(text);

        let movies = d.find(Class("mb-4")).skip(2).collect::<Vec<_>>();
        for m in movies {
            let title = match
                m.find(Class("mc-title")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Title: '<div>' not found.");
                    return;
                }
                Some(div) =>
                    match div.find(Class("d-md-none")).collect::<Vec<_>>().first() {
                        None => {
                            println!("[ERROR] Title: '<a>' not found.");
                            return;
                        },
                        Some(a) => a.text(),
                    }
            };

            let year = match
                m.find(Class("mc-year")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Year: '<span>' not found.");
                    return;
                }
                Some(span) => span.text(),
            };

            let directors = match
                m.find(Class("credits")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Year: '<span>' not found.");
                    return;
                }
                Some(div) => div.find(Name("a")).map(|a| a.text()).collect::<Vec<_>>()
            };

            let rating: String = match
                m.find(Class("fa-user-rat-box")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Year: '<span>' not found.");
                    return;
                }
                Some(div) => match div.text().trim().parse::<i32>() {
                    Err(e) => {
                        // println!("{} - {}", e, title);
                        // panic!("FUCK")
                        "".to_owned()
                    },
                    Ok(f) => (f).to_string()
                },
            };

            // println!("rating: {}", rating);
            data.push(format!("\"{}\",{},\"{}\",{}",
                              title,
                              year,
                              directors.iter().format(","),
                              rating
            ));
            // println!("{},{},\"{}\",{:.1}", title, year, directors.iter().format(","), rating)
        }
    }

    std::thread::sleep(time::Duration::from_secs_f64(1.5));

    let mut content = String::with_capacity(10000);
    for d in data {
        content.push_str(&d);
        content.push('\n');
    }

    fs::write(
        std::path::Path::new("result.csv"),
        content
    ).unwrap();
}