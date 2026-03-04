//! Version checker: non-blocking startup check for newer crate versions.
//!
//! Spawns a background task that queries crates.io. If `koda-cli` exists,
//! nudges the user to migrate. Otherwise falls back to `koda-agent` updates.

use std::time::Duration;

const CRATES_IO_AGENT_URL: &str = "https://crates.io/api/v1/crates/koda-agent";
const CRATES_IO_CLI_URL: &str = "https://crates.io/api/v1/crates/koda-cli";
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// Spawn a background version check. Returns a handle that can be awaited.
pub fn spawn_version_check() -> tokio::task::JoinHandle<Option<String>> {
    tokio::spawn(async move { check_latest_version().await })
}

/// Print the update hint.
///
/// If `koda-cli` is published on crates.io, always nudge migration regardless
/// of version comparison. Otherwise fall back to `koda-agent` update hint.
pub fn print_update_hint(latest: &str) {
    // `latest` is formatted as "cli:VERSION" or a bare version string.
    if let Some(cli_ver) = latest.strip_prefix("cli:") {
        println!(
            "  \x1b[90m\u{2728} \x1b[0m\x1b[33mkoda-agent is deprecated.\x1b[0m\x1b[90m Migrate to \x1b[0m\x1b[32mkoda-cli v{cli_ver}\x1b[0m\x1b[90m  (cargo install koda-cli)\x1b[0m"
        );
        println!();
        return;
    }

    let current = env!("CARGO_PKG_VERSION");
    if latest != current && is_newer(latest, current) {
        println!(
            "  \x1b[90m\u{2728} Update available: \x1b[0m\x1b[36m{current}\x1b[0m\x1b[90m \u{2192} \x1b[0m\x1b[32m{latest}\x1b[0m\x1b[90m  (cargo install koda-agent)\x1b[0m"
        );
        println!();
    }
}

/// Query crates.io — prefer `koda-cli` if published, else check `koda-agent`.
async fn check_latest_version() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(CHECK_TIMEOUT)
        .build()
        .ok()?;

    let ua = format!("Koda/{} (version-check)", env!("CARGO_PKG_VERSION"));

    // First, check if koda-cli exists on crates.io
    if let Some(cli_ver) = query_crate(&client, CRATES_IO_CLI_URL, &ua).await {
        return Some(format!("cli:{cli_ver}"));
    }

    // Fallback: check koda-agent for a newer version
    query_crate(&client, CRATES_IO_AGENT_URL, &ua).await
}

async fn query_crate(client: &reqwest::Client, url: &str, ua: &str) -> Option<String> {
    let resp = client.get(url).header("User-Agent", ua).send().await.ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let body: serde_json::Value = resp.json().await.ok()?;
    body.get("crate")?
        .get("max_version")?
        .as_str()
        .map(|s| s.to_string())
}

/// Simple semver comparison: is `a` newer than `b`?
fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
    let va = parse(a);
    let vb = parse(b);
    va > vb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_is_newer_same_version() {
        assert!(!is_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_print_update_hint_cli_prefix() {
        // Just verify the prefix parsing logic works
        let latest = "cli:0.1.0";
        assert_eq!(latest.strip_prefix("cli:"), Some("0.1.0"));
    }

    #[test]
    fn test_print_update_hint_bare_version() {
        let latest = "0.2.0";
        assert!(latest.strip_prefix("cli:").is_none());
    }
}
