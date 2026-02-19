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
