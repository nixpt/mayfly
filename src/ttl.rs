use anyhow::{bail, Result};
use std::time::Duration;

/// Parse `15m` / `30s` / `2h` style durations.
pub fn parse_ttl(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty ttl");
    }
    let (num, unit) = s.split_at(s.len() - 1);
    let n: u64 = num.parse().map_err(|_| anyhow::anyhow!("bad ttl number in {s}"))?;
    Ok(match unit {
        "s" => Duration::from_secs(n),
        "m" => Duration::from_secs(n * 60),
        "h" => Duration::from_secs(n * 3600),
        _ => bail!("ttl unit must be s|m|h, got {s}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        assert_eq!(parse_ttl("15m").unwrap(), Duration::from_secs(900));
        assert_eq!(parse_ttl("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_ttl("2h").unwrap(), Duration::from_secs(7200));
    }
}
