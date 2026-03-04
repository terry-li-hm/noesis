use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod display;
mod log;

#[derive(Parser)]
#[command(
    name = "noesis",
    version,
    about = "Perplexity API CLI — search, ask, research, and reason from your terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Print raw JSON response instead of extracted content
    #[arg(long, global = true)]
    raw: bool,

    /// Skip logging this query
    #[arg(long, global = true)]
    no_log: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Quick search (sonar, ~$0.006)
    Search {
        /// The query to search for
        query: String,
    },
    /// Pro search (sonar-pro, ~$0.01)
    Ask {
        /// The query to ask
        query: String,
    },
    /// Deep research (sonar-deep-research, ~$0.40) — EXPENSIVE
    Research {
        /// The query to research
        query: String,
    },
    /// Reasoning (sonar-reasoning-pro, ~$0.01)
    Reason {
        /// The query to reason about
        query: String,
    },
    /// Show usage log
    Log {
        /// Show all entries (default: last 20)
        #[arg(long)]
        all: bool,
        /// Show summary statistics
        #[arg(long)]
        stats: bool,
    },
}

fn mode_and_model(cmd: &Command) -> Option<(&'static str, &'static str, f64)> {
    match cmd {
        Command::Search { .. } => Some(("search", "sonar", 0.006)),
        Command::Ask { .. } => Some(("ask", "sonar-pro", 0.01)),
        Command::Research { .. } => Some(("research", "sonar-deep-research", 0.40)),
        Command::Reason { .. } => Some(("reason", "sonar-reasoning-pro", 0.01)),
        Command::Log { .. } => None,
    }
}

fn query_text(cmd: &Command) -> Option<&str> {
    match cmd {
        Command::Search { query }
        | Command::Ask { query }
        | Command::Research { query }
        | Command::Reason { query } => Some(query),
        Command::Log { .. } => None,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Command::Log { all, stats } = &cli.command {
        if *stats {
            log::display_stats()?;
        } else {
            log::display_log(*all)?;
        }
        return Ok(());
    }

    let (mode, model, est_cost) = mode_and_model(&cli.command).unwrap();
    let query = query_text(&cli.command).unwrap();

    let client = client::PplxClient::new()?;
    let start = std::time::Instant::now();
    let response = client.query(model, query)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    if cli.raw {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        display::display_response(mode, &response);
    }

    if !cli.no_log {
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");
        log::append(mode, model, query, content.len(), est_cost, duration_ms)?;
    }

    Ok(())
}
