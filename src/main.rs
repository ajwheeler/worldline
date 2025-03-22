use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(author, version, about = "Manipulate the worldline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new event to the timeline
    #[command(about = "Add a new event with date and description", alias = "a")]
    Add { date: String, description: String },

    /// Display events from the timeline
    #[command(
        about = "Show events. No args = show all. One date = show that date/month/year. Two dates = show range",
        alias = "s"
    )]
    Show {
        #[arg(num_args = 0..=2)]
        dates: Vec<String>,
    },

    /// Search for events
    #[command(
        about = "Search for events containing text (case-insensitive)",
        alias = "q"
    )]
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

    let worldline_file = match env::var("WORLDLINE_FILE") {
        Ok(filename) => filename,
        Err(e) => {
            eprintln!(
                "Could not read the WORLDLINE_FILE environment variable: {}",
                e
            );
            std::process::exit(1);
        }
    };

    let mut worldline = match wl::WorldLine::from_file(&worldline_file) {
        Ok(worldline) => worldline,
        Err(e) => {
            eprintln!("Error: Could not read worldline file: {}", e);
            eprintln!("Expected to find a worldline file at {}", worldline_file);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Add { date, description } => {
            let event = wl::Event::new(parse_date(&date), description);
            let idx = worldline.add_event(event);
            let lb = std::cmp::max(0, idx - 1);
            let ub = std::cmp::min(worldline.len(), idx + 2);
            if let Err(e) = worldline.to_file(&worldline_file) {
                eprintln!("Warning: Could not write worldline file: {}", e);
            }
            worldline.print_range(lb, ub);
        }
        Commands::Show { dates } => {
            if dates.is_empty() {
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
