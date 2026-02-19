use chrono::DateTime;

pub fn date<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
    let s = s.to_string();

    let dt = DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S %:z")
        .map_err(|e| ::askama::Error::Custom(Box::new(e)))?;

    let offset = dt.offset().local_minus_utc();
    let hours = offset / 3600;
    let minutes = (offset % 3600).abs() / 60;

    let tz_str = if minutes == 0 {
        format!("{:+}", hours)
    } else {
        format!("{:+}:{:02}", hours, minutes)
    };

    let formatted = format!("{} {}", dt.format("%Y/%m/%d %H:%M"), tz_str);

    Ok(formatted)
}

pub fn truncate<T: std::fmt::Display>(s: T, len: usize) -> ::askama::Result<String> {
    let s = s.to_string();
    Ok(if s.chars().count() > len {
        format!("{}...", s.chars().take(len).collect::<String>())
    } else {
        s
    })
}

pub fn lower<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
    Ok(s.to_string().to_lowercase())
}

pub fn upper<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
    Ok(s.to_string().to_uppercase())
}

pub fn replace<T: std::fmt::Display>(s: T, from: &str, to: &str) -> ::askama::Result<String> {
    Ok(s.to_string().replace(from, to))
}

pub fn to_ref(s: &u32) -> ::askama::Result<&u32> {
    Ok(s)
}

pub fn percentage(current: &usize, total: &usize) -> ::askama::Result<usize> {
    if *total == 0 {
        Ok(0)
    } else {
        Ok((current * 100) / total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_with_timezone() {
        let result = date("2024-06-15 14:30:00 +09:00").unwrap();
        assert_eq!(result, "2024/06/15 14:30 +9");
    }

    #[test]
    fn test_date_utc() {
        let result = date("2024-01-01 00:00:00 +00:00").unwrap();
        assert_eq!(result, "2024/01/01 00:00 +0");
    }

    #[test]
    fn test_date_negative_offset() {
        let result = date("2024-03-10 08:00:00 -05:00").unwrap();
        assert_eq!(result, "2024/03/10 08:00 -5");
    }

    #[test]
    fn test_date_half_hour_offset() {
        let result = date("2024-01-01 00:00:00 +05:30").unwrap();
        assert_eq!(result, "2024/01/01 00:00 +5:30");
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10).unwrap(), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5).unwrap(), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 5).unwrap(), "hello...");
    }

    #[test]
    fn test_truncate_korean() {
        assert_eq!(truncate("안녕하세요", 3).unwrap(), "안녕하...");
    }

    #[test]
    fn test_truncate_emoji() {
        assert_eq!(truncate("🎉🎊🎈🎁", 2).unwrap(), "🎉🎊...");
    }

    #[test]
    fn test_percentage_normal() {
        assert_eq!(percentage(&50, &100).unwrap(), 50);
    }

    #[test]
    fn test_percentage_zero_total() {
        assert_eq!(percentage(&0, &0).unwrap(), 0);
    }

    #[test]
    fn test_percentage_full() {
        assert_eq!(percentage(&100, &100).unwrap(), 100);
    }

    #[test]
    fn test_percentage_nonzero_over_zero() {
        assert_eq!(percentage(&5, &0).unwrap(), 0);
    }
}
