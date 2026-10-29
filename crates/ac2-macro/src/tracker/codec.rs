//! `patterns.txt`: a count line, then that many patterns, CRLF-separated (Virindi Sense's format,
//! so existing lists load unchanged).

pub(super) fn decode_patterns(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines();
    let Some(count) = lines
        .next()
        .and_then(|line| line.trim().parse::<usize>().ok())
    else {
        return Vec::new();
    };
    lines.take(count).map(str::to_owned).collect()
}

pub(super) fn encode_patterns<'a>(patterns: impl IntoIterator<Item = &'a str>) -> Vec<u8> {
    let patterns: Vec<&str> = patterns.into_iter().collect();
    let mut text = patterns.len().to_string();
    text.push_str("\r\n");
    for pattern in patterns {
        text.push_str(pattern);
        text.push_str("\r\n");
    }
    text.into_bytes()
}
