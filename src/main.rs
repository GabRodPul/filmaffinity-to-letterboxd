use std::fs;
use fantoccini::Client;
use itertools::Itertools;
use select::{
    document::Document,
    predicate::{Class, Name}
};

use clap::{Parser};

/// Arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ID of the user to scrape
    #[arg(short, long)]
    user_id: u32,

    /// Number of pages to scrape
    #[arg(short('c'), long)]
    page_count: usize,

    /// Chromedriver/Geckodriver port
    #[arg(short('p'), long)]
    webdriver_port: u16,

    /// Output file name
    #[arg(short, long, default_value_t = String::from("result.csv"))]
    output_file: String,

    /// Saves the file when an element isn't found
    #[arg(short, long, default_value_t = true)]
    save_on_not_found: bool
}

// Fn's
const CSV_HEADER: &str = "Title,Year,Directors,Rating10";
fn save_csv(data: Vec<String>, out_str: &str) {
    if let Some(d) = data.last() && d != CSV_HEADER {
        println!("Saving a total of {} movies", data.len())
    } else {
        println!("Data empty when saving .csv - file will be written anyway.");
    }

    let mut content = String::with_capacity(10000);
    for d in data {
        content.push_str(&d);
        content.push('\n');
    }

    fs::write(
        std::path::Path::new(out_str),
        content
    ).unwrap();
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let webdriver_url: &str = &format!("http://localhost:{}", args.webdriver_port);
    use fantoccini::{ClientBuilder};
    let c = ClientBuilder::native().connect(webdriver_url).await.unwrap();
    
    scrape(args, &c).await;
    c.close().await.unwrap();
}

const STRUCTURE_CHANGE: &str = "FilmAffinity's HTML structure has changed and this repository needs updating";
async fn scrape(args: Args, c: &Client) {
    let base_url: &str = &format!("https://www.filmaffinity.com/en/userratings.php?user_id={}&orderby=4", args.user_id);
    let mut data = Vec::<String>::with_capacity(50 * args.page_count);
    data.push(CSV_HEADER.to_owned());

    for p in 1..=args.page_count {
        let url = format!("{}&p={}&chv=list", base_url, p);
        println!("Processing URL: {}", url);
        
        // FilmAffinity needs JS to do it's thing in order to display content
        // Plain HTML requests won't work
        c.goto(&url).await.unwrap();
        
        let text: String = match c.source().await {
            Err(e) => {
                println!("{}", e);
                return;
            },
            Ok(r) => r
        };

        let d = Document::from(&text[..]);
        if let Some(_) = text.find("<h1>Not Found</h1>") {
            println!("[ERROR] User with id {} not found", args.user_id);
            return;
        }
        drop(text);

        let movies = d.find(Class("mb-4")).skip(2).collect::<Vec<_>>();
        if movies.is_empty() {
            println!("[ERROR] {}", STRUCTURE_CHANGE);
            return;
        }

        for m in movies {
            let title = match
                m.find(Class("mc-title")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Title: '<div>' not found. {}", STRUCTURE_CHANGE);
                    if args.save_on_not_found { save_csv(data, &args.output_file); }
                    return;
                }   
                Some(div) =>
                    match div.find(Class("d-md-none")).collect::<Vec<_>>().first() {
                        None => {
                            println!("[ERROR] Title: '<a>' not found. {}", STRUCTURE_CHANGE);
                            if args.save_on_not_found { save_csv(data, &args.output_file); }
                            return;
                        },
                        Some(a) => a.text(),
                    }
            };

            let year = match
                m.find(Class("mc-year")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Year: '<span>' not found. {}", STRUCTURE_CHANGE);
                    if args.save_on_not_found { save_csv(data, &args.output_file); }
                    return;
                }
                Some(span) => span.text(),
            };

            let directors = match
                m.find(Class("credits")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Director: '<span>' not found. {}", STRUCTURE_CHANGE);
                    if args.save_on_not_found { save_csv(data, &args.output_file); }
                    return;
                }
                Some(div) => div.find(Name("a")).map(|a| a.text()).collect::<Vec<_>>()
            };

            let rating: String = match
                m.find(Class("fa-user-rat-box")).collect::<Vec<_>>().first()
            {
                None => {
                    println!("[ERROR] Rating: '<span>' not found. {}", STRUCTURE_CHANGE);
                    if args.save_on_not_found { save_csv(data, &args.output_file); }
                    return;
                }
                Some(div) => match div.text().trim().parse::<i32>() {
                    Err(_) => {
                        "".to_owned()
                    },
                    Ok(f) => (f).to_string()
                },
            };

            data.push(format!("\"{}\",{},\"{}\",{}",
                              title,
                              year,
                              directors.iter().format(","),
                              rating
            ));
        }
    }

    save_csv(data, &args.output_file);
}