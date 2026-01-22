use anyhow::{Result, bail};
use chaser_oxide::{Browser, BrowserConfig, ChaserPage, ChaserProfile};
use clap::{ArgAction, Parser};
use core::fmt;
use fancy_log::{LogLevel, log, set_log_level};
use futures::StreamExt;
use itertools::Itertools;
use select::{
    document::Document,
    predicate::{Class, Name}
};
use std::{error::Error, fs, time::Duration};

/// Structs
const DEF_PAGE_COUNT: usize = 1000;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ID of the user to scrape
    #[arg(short, long)]
    user_id: u32,

    /// Number of pages to scrape
    #[arg(short, long, default_value_t = DEF_PAGE_COUNT)]
    page_count: usize,

    /// Output file name
    #[arg(short, long, default_value_t = String::from("filmaffinity-to-letterboxd-result.csv"))]
    output_file: String,
    
    /// If true, will delay requests after HTML processing by a random value within the integral range [1, 3]
    #[arg(short('d'), long, default_value_t = true, action = ArgAction::SetFalse)]
    // TODO: Consider adding custom delay times
    use_delay: bool,
}

#[derive(Debug, Clone)]
enum ScrapingError {
    UserNotFound(u32),
    StructureChange { name: &'static str, element: &'static str },
    Cloudflare
}

impl fmt::Display for ScrapingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UserNotFound(id) => write!(f, "{}", format!("User with id {} not found", id)),
            Self::StructureChange{ name, element } 
                => write!(f, "{}", format!("{}: '{}' not found — FilmAffinity's HTML structure has changed", name, element)),
            Self::Cloudflare 
                => write!(f, "WebDriver is being blocked by Cloudflare")
        }
    }
}

impl Error for ScrapingError { }


// Fn's
const CSV_HEADER: &str = "Title,Year,Directors,Rating10";
fn save_csv(data: Vec<String>, out_str: &str) {
    if let Some(d) = data.last() && d != CSV_HEADER {
        log(LogLevel::Info, &format!("[INFO] Saving a total of {} movies", data.len()));
    } else {
        log(LogLevel::Info, "[INFO] Data empty when saving .csv - file will be written anyway.");
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
    set_log_level(LogLevel::Info);
    match scrape(args).await {
        Err(e) => log(LogLevel::Error, &format!("[ERROR] {}", e)),
        _      => ()
    }
}

async fn scrape(args: Args) -> Result<()> {
    let base_url: &str = &format!("https://www.filmaffinity.com/en/userratings.php?user_id={}&orderby=4", args.user_id);
    let mut data = Vec::<String>::with_capacity(50 * args.page_count);
    data.push(CSV_HEADER.to_owned());

    // FilmAffinity needs JS to do it's thing in order to display content
    // Plain HTML requests won't work
    let profile = ChaserProfile::windows().build();
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder().new_headless_mode().build().unwrap()
    ).await.unwrap();

    tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    let c = ChaserPage::new(browser.new_page("about:blank").await.unwrap());
    c.apply_profile(&profile).await.unwrap();

    let delays: Vec<u8> = if args.use_delay {
        log(LogLevel::Warn, "[WARNING] DELAY: The app will delay the next request after processing all data by a random integral range of [1, 3]");
        log(LogLevel::Warn, "[WARNING] DELAY: Although as far as testing goes, no delay hasn't caused any issues, better to be safe than sorry");
        log(LogLevel::Warn, "[WARNING] DELAY: To disable the delay, add `-d` to the flags");

        let mut rng = fastrand::Rng::new();
        std::iter::repeat_with(|| rng.u8(1..=3))
            .take(args.page_count)
            .collect()
    } else {
        vec![]
    };
    
    for p in 1..=args.page_count {
        let url = format!("{}&p={}&chv=list", base_url, p);
        log(LogLevel::Info, &format!("[INFO] Processing page nº{:>4} for user {}: {}", p, args.user_id, url));
        
        c.goto(&url).await.unwrap();
        let text: String = c.content().await?;

        let d = Document::from(&text[..]);
        if let Some(_) = text.find("<h1>Not Found</h1>") {
            if p == 1 {
                bail!(ScrapingError::UserNotFound(args.user_id))
            } else {
                log(LogLevel::Warn, &format!("[WARNING] Page nº {} not found; max `page_count` was {}{}", p, args.page_count, 
                    if args.page_count == DEF_PAGE_COUNT { " (default)" } else { "" }
                ));

                save_csv(data, &args.output_file);
                return Ok(())
            }
        }
        drop(text);

        let movies = d.find(Class("mb-4")).skip(2).collect::<Vec<_>>();
        if movies.is_empty() {
            if data.is_empty() {
                bail!(ScrapingError::StructureChange {
                    name:       "Movies", 
                    element:    "<div class=\".. mb-4\">",
                })
            } else { 
                // The existence of previous data means we're getting blocked by Cloudflare
                bail!(ScrapingError::Cloudflare)
            }
        }

        for m in movies {
            let title = match
                m.find(Class("mc-title")).collect::<Vec<_>>().first()
            {
                None => bail!(ScrapingError::StructureChange { 
                    name:    "Title", 
                    element: "<div class=\".. mc-title\">" 
                }),
                Some(div) =>
                    match div.find(Class("d-md-none")).collect::<Vec<_>>().first() {
                        None => bail!(ScrapingError::StructureChange { 
                            name:    "Title", 
                            element: "<a class=\".. d-md-none\">" 
                        }),
                        Some(a) => a.text(),
                    }
            };

            let year = match
                m.find(Class("mc-year")).collect::<Vec<_>>().first()
            {
                None => bail!(ScrapingError::StructureChange { 
                    name:    "Year", 
                    element: "<span class=\".. mc-year\">"
                }),
                Some(span) => span.text(),
            };

            let directors = match
                m.find(Class("credits")).collect::<Vec<_>>().first()
            {
                None => bail!(ScrapingError::StructureChange { 
                    name:    "Director", 
                    element: "<span class=\".. credits\">"
                }),
                Some(div) => div.find(Name("a")).map(|a| a.text()).collect::<Vec<_>>()
            };

            let rating: String = match
                m.find(Class("fa-user-rat-box")).collect::<Vec<_>>().first()
            {
                None => bail!(ScrapingError::StructureChange {
                    name:    "Rating", 
                    element: "<span class=\".. fa-user-rat-box\">" 
                }),
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

        if args.use_delay {
            std::thread::sleep(Duration::from_secs(delays[p-1] as u64));
        }
    }

    save_csv(data, &args.output_file);
    Ok(())
}