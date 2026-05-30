use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub name: String,
    pub country: String,
    pub city: String,
    pub ping: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VpnStatus {
    pub connected: bool,
    pub location: Option<String>,
    pub raw: String,
}

/// Strip ANSI escape codes from text
fn strip_ansi_codes(text: &str) -> String {
    let ansi_regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    ansi_regex.replace_all(text, "").to_string()
}

async fn run_cli(cli_path: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cli_path).args(args).output().await?;
    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(strip_ansi_codes(&raw))
}

pub async fn get_status(cli_path: &str) -> Result<VpnStatus> {
    let output = run_cli(cli_path, &["status"]).await?;
    let lower = output.to_lowercase();
    let connected = lower.contains("connected") && !lower.contains("disconnected");
    let location = parse_location_from_status(&output);
    Ok(VpnStatus {
        connected,
        location,
        raw: output,
    })
}

fn parse_location_from_status(output: &str) -> Option<String> {
    for line in output.lines() {
        let lower = line.to_lowercase();
        // Look for "Connected to <location>" pattern
        if lower.contains("connected to") {
            if let Some(start) = line.to_lowercase().find("connected to") {
                let after = line[start + "connected to".len()..].trim();
                // Extract up to " in" or end of line
                let location = if let Some(end) = after.to_lowercase().find(" in ") {
                    after[..end].trim().to_string()
                } else {
                    after.to_string()
                };
                if !location.is_empty() {
                    return Some(location);
                }
            }
        }
        // Fallback for other formats with colons (but skip IP addresses)
        if lower.contains("location") || lower.contains("server") {
            if let Some(pos) = line.find(':') {
                let value = line[pos + 1..].trim().to_string();
                if !value.is_empty() && !value.contains('.') {
                    return Some(value);
                }
            }
        }
    }
    None
}

pub async fn list_locations(cli_path: &str) -> Result<Vec<Location>> {
    let output = run_cli(cli_path, &["list-locations"]).await?;
    parse_locations(&output)
}

fn parse_locations(output: &str) -> Result<Vec<Location>> {
    let mut locations = Vec::new();
    let separator = Regex::new(r"\s{2,}").unwrap();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('-') || trimmed.starts_with('=') {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("iso")
            || lower.contains("country")
            || lower.contains("city")
            || lower.contains("ping")
        {
            continue;
        }

        let fields: Vec<&str> = separator.split(trimmed).collect();
        if fields.len() >= 3 {
            let name = fields[0].trim().to_string();
            let country = fields[1].trim().to_string();
            let city = fields[2].trim().to_string();
            let ping = fields.get(3).map(|s| s.trim().to_string());

            locations.push(Location {
                name,
                country,
                city,
                ping,
            });
        }
    }

    Ok(locations)
}

pub async fn connect(cli_path: &str, location: &str) -> Result<()> {
    let output = Command::new(cli_path)
        .args(["connect", "-l", location, "-y"])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(anyhow!(
            "connect failed: {}{}",
            stdout.trim(),
            stderr.trim()
        ))
    }
}

pub async fn disconnect(cli_path: &str) -> Result<()> {
    let output = Command::new(cli_path).arg("disconnect").output().await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("disconnect failed: {}", stderr.trim()))
    }
}
