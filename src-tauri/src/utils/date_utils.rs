use chrono::{DateTime, Utc, Duration};

/// Format date time as user-friendly string
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Check if token is expired
pub fn is_token_expired(expires_at: &DateTime<Utc>) -> bool {
    Utc::now() >= *expires_at
}

/// Check if token is expiring soon (within 5 minutes)
pub fn is_token_expiring_soon(expires_at: &DateTime<Utc>) -> bool {
    Utc::now() + Duration::minutes(5) >= *expires_at
}

/// Calculate duration from now to target time (in seconds)
pub fn seconds_until(target: &DateTime<Utc>) -> i64 {
    (*target - Utc::now()).num_seconds()
}

/// Parse ISO 8601 format time string
pub fn parse_iso_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    s.parse::<DateTime<Utc>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_datetime() {
        let dt = "2024-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(format_datetime(&dt), "2024-01-01 12:00:00");
    }

    #[test]
    fn test_is_token_expired() {
        let past = Utc::now() - Duration::hours(1);
        let future = Utc::now() + Duration::hours(1);
        
        assert!(is_token_expired(&past));
        assert!(!is_token_expired(&future));
    }
}
