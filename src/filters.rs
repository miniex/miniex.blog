use chrono::NaiveDateTime;

pub fn date<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
    let s = s.to_string();

    let dt = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S UTC")
        .expect("Failed to parse datetime");

    let formatted = dt.format("%Y/%m/%d %H:%M").to_string();

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

pub fn replace<T: std::fmt::Display>(s: T, from: &str, to: &str) -> ::askama::Result<String> {
    Ok(s.to_string().replace(from, to))
}
