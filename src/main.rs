use clap::{Parser, Subcommand};
use std::env;

use wl;

#[derive(Parser)]
#[command(author, version, about = "Manipulate the worldline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "a")]
    Add { date: String, description: String },
    #[command(alias = "s")]
    Show {
        #[arg(num_args = 0..=2)]
        dates: Vec<String>,
    },
    #[command(alias = "q")]
    Query { query: String },
}

fn parse_date(date_str: &str) -> wl::Date {
    wl::Date::parse(date_str)
        .unwrap_or_else(|_| {
            eprintln!("Error: Could not parse date '{}'", date_str);
            std::process::exit(1);
        })
        .0
}

fn main() {
    let cli = Cli::parse();

    let worldline_file =
        env::var("WORLDLINE_FILE").expect("WORLDLINE_FILE environment variable not set");
    let mut worldline =
        wl::WorldLine::from_file(&worldline_file).expect("Could not read worldline file");

    match cli.command {
        Commands::Add { date, description } => {
            let event = wl::Event::new(parse_date(&date), description);
            let idx = worldline.add_event(event);
            let lb = std::cmp::max(0, idx - 1);
            let ub = std::cmp::min(worldline.len(), idx + 2);
            worldline
                .to_file(&worldline_file)
                .expect("Could not write worldline file");
            worldline.print_range(lb, ub);
        }
        Commands::Show { dates } => {
            if dates.len() == 0 {
                worldline.print_all();
            } else if dates.len() == 1 {
                let date = parse_date(&dates[0]);
                worldline.print_implicit_date_range(date);
            } else if dates.len() == 2 {
                let start = parse_date(&dates[0]);
                let end = parse_date(&dates[1]);
                worldline.print_date_range(start, end);
            }
        }
        Commands::Query { query } => {
            worldline.query_and_print(&query);
        }
    }
}
