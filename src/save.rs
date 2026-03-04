use anyhow::{Context, Result};
use chrono::Local;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn slugify(query: &str) -> String {
    let slug: String = query
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse runs of dashes and trim
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    // Truncate to 60 chars at a word boundary
    if slug.len() <= 60 {
        slug
    } else {
        let truncated = &slug[..60];
        match truncated.rfind('-') {
            Some(pos) => truncated[..pos].to_string(),
            None => truncated.to_string(),
        }
    }
}

fn save_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join("docs/solutions/research");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn save_research(query: &str, response: &Value, est_cost: f64) -> Result<PathBuf> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let slug = slugify(query);
    let filename = format!("{date}-{slug}.md");
    let path = save_dir()?.join(&filename);

    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    let citations: Vec<&str> = response["citations"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut sources_yaml = String::new();
    for url in &citations {
        sources_yaml.push_str(&format!("  - {url}\n"));
    }

    let mut doc = format!(
        "---\nquery: \"{query}\"\ndate: {date}\nmodel: sonar-deep-research\ncost_est: ${est_cost:.2}\nsources:\n{sources_yaml}---\n\n# {query}\n\n{content}\n"
    );

    if !citations.is_empty() {
        doc.push_str("\n## Sources\n");
        for (i, url) in citations.iter().enumerate() {
            doc.push_str(&format!("{}. {}\n", i + 1, url));
        }
    }

    fs::write(&path, doc)?;
    Ok(path)
}
