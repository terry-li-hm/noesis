use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LogEntry {
    ts: String,
    mode: String,
    model: String,
    query: String,
    response_len: usize,
    est_cost_usd: f64,
    duration_ms: u64,
}

fn log_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join("noesis");
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("log.jsonl"))
}

pub fn append(
    mode: &str,
    model: &str,
    query: &str,
    response_len: usize,
    est_cost_usd: f64,
    duration_ms: u64,
) -> Result<()> {
    let entry = LogEntry {
        ts: Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        mode: mode.to_string(),
        model: model.to_string(),
        query: query.to_string(),
        response_len,
        est_cost_usd,
        duration_ms,
    };

    let path = log_path()?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("Failed to open log file")?;

    let line = serde_json::to_string(&entry)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn read_entries() -> Result<Vec<LogEntry>> {
    let path = log_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub fn display_log(all: bool) -> Result<()> {
    let entries = read_entries()?;
    if entries.is_empty() {
        println!("  No log entries yet.");
        return Ok(());
    }

    let show: &[LogEntry] = if all || entries.len() <= 20 {
        &entries
    } else {
        &entries[entries.len() - 20..]
    };

    if !all && entries.len() > 20 {
        eprintln!(
            "{}",
            format!("  (showing last 20 of {} — use --all for full log)", entries.len()).dimmed()
        );
    }

    for e in show {
        let ts = &e.ts[..16]; // "2026-02-14T15:30"
        let mode_colored = match e.mode.as_str() {
            "search" => e.mode.cyan().to_string(),
            "ask" => e.mode.green().to_string(),
            "research" => e.mode.magenta().to_string(),
            "reason" => e.mode.yellow().to_string(),
            _ => e.mode.clone(),
        };
        let query_short = if e.query.len() > 60 {
            format!("{}…", &e.query[..59])
        } else {
            e.query.clone()
        };
        let cost = format!("${:.3}", e.est_cost_usd);
        let dur = format!("{:.1}s", e.duration_ms as f64 / 1000.0);
        println!(
            "  {} {:<10} {:>6} {:>5}  {}",
            ts.dimmed(),
            mode_colored,
            cost.dimmed(),
            dur.dimmed(),
            query_short,
        );
    }

    Ok(())
}

pub fn display_stats() -> Result<()> {
    let entries = read_entries()?;
    if entries.is_empty() {
        println!("  No log entries yet.");
        return Ok(());
    }

    let mut by_mode: HashMap<String, (usize, f64)> = HashMap::new();
    let mut total_cost = 0.0;

    for e in &entries {
        let entry = by_mode.entry(e.mode.clone()).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += e.est_cost_usd;
        total_cost += e.est_cost_usd;
    }

    let first_ts = &entries.first().unwrap().ts;
    let last_ts = &entries.last().unwrap().ts;

    println!("  {} queries from {} to {}", entries.len(), &first_ts[..10], &last_ts[..10]);
    println!();

    let mut modes: Vec<_> = by_mode.into_iter().collect();
    modes.sort_by_key(|(m, _)| match m.as_str() {
        "search" => 0,
        "ask" => 1,
        "reason" => 2,
        "research" => 3,
        _ => 4,
    });

    println!("  {:<12}{:>8}{:>10}", "Mode".dimmed(), "Count".dimmed(), "Cost".dimmed());
    for (mode, (count, cost)) in &modes {
        let mode_colored = match mode.as_str() {
            "search" => mode.cyan().to_string(),
            "ask" => mode.green().to_string(),
            "research" => mode.magenta().to_string(),
            "reason" => mode.yellow().to_string(),
            _ => mode.clone(),
        };
        println!("  {:<12}{:>8}{:>10}", mode_colored, count, format!("${:.2}", cost));
    }

    println!("  {}", "─".repeat(30).dimmed());
    println!("  {:<12}{:>8}{:>10}", "Total".bold(), entries.len(), format!("${:.2}", total_cost).bold());

    Ok(())
}
