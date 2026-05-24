pub fn normalize_base_url(value: &str) -> String {
    value.trim().trim_end_matches("/").to_string()
}

pub fn has_supported_url_scheme(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

pub fn services_url(base_url: &str) -> String {
    format!("{}/api/getallservices", normalize_base_url(base_url))
}

pub fn epg_now_url(base_url: &str, bouquet_ref: &str) -> String {
    format!(
        "{}/api/epgnow?bRef={}",
        normalize_base_url(base_url),
        encode_query_value(bouquet_ref)
    )
}

pub fn stream_m3u_url(base_url: &str, service_ref: &str) -> String {
    format!(
        "{}/web/stream.m3u?ref={}",
        normalize_base_url(base_url),
        encode_query_value(service_ref)
    )
}

pub fn encode_query_value(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub fn extract_stream_url(m3u: &str) -> Option<String> {
    m3u.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("#"))
        .map(ToString::to_string)
}
