use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bouquet {
    pub name: String,
    pub service_ref: String,
    pub channels: Vec<Channel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    pub position: u32,
    pub name: String,
    pub service_ref: String,
    pub program: Option<u32>,
    pub epg: Option<EpgEvent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct EpgEvent {
    pub id: Option<u64>,
    #[serde(default)]
    pub begin_timestamp: i64,
    #[serde(default)]
    pub duration_sec: i64,
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub shortdesc: String,
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub longdesc: String,
    #[serde(default)]
    pub sref: String,
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub sname: String,
    #[serde(default)]
    pub now_timestamp: i64,
    #[serde(default)]
    pub remaining: i64,
}

#[derive(Debug, Deserialize)]
pub struct ServicesResponse {
    #[serde(default)]
    pub result: bool,
    #[serde(default)]
    pub services: Vec<ApiBouquet>,
}

#[derive(Debug, Deserialize)]
pub struct ApiBouquet {
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub servicename: String,
    #[serde(default)]
    pub servicereference: String,
    #[serde(default)]
    pub subservices: Vec<ApiChannel>,
}

#[derive(Debug, Deserialize)]
pub struct ApiChannel {
    #[serde(default)]
    pub pos: u32,
    #[serde(default, deserialize_with = "deserialize_normalized_text")]
    pub servicename: String,
    #[serde(default)]
    pub servicereference: String,
    pub program: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EpgNowResponse {
    #[serde(default)]
    pub events: Vec<EpgEvent>,
}

impl From<ApiBouquet> for Bouquet {
    fn from(value: ApiBouquet) -> Self {
        Self {
            name: value.servicename,
            service_ref: value.servicereference,
            channels: value.subservices.into_iter().map(Channel::from).collect(),
        }
    }
}

impl From<ApiChannel> for Channel {
    fn from(value: ApiChannel) -> Self {
        Self {
            position: value.pos,
            name: value.servicename,
            service_ref: value.servicereference,
            program: value.program,
            epg: None,
        }
    }
}

impl EpgEvent {
    pub fn progress(&self) -> f64 {
        if self.duration_sec <= 0 {
            return 0.0;
        }

        let elapsed = self
            .now_timestamp
            .saturating_sub(self.begin_timestamp)
            .clamp(0, self.duration_sec);
        elapsed as f64 / self.duration_sec as f64
    }

    pub fn time_range(&self) -> String {
        format!(
            "{} - {}",
            format_time(self.begin_timestamp),
            format_time(self.end_timestamp())
        )
    }

    pub fn end_timestamp(&self) -> i64 {
        self.begin_timestamp
            .saturating_add(self.duration_sec.max(0))
    }

    pub fn description(&self) -> String {
        let short = normalize_epg_text(&self.shortdesc);
        let long = normalize_epg_text(&self.longdesc);
        match (short.as_str(), long.as_str()) {
            ("", "") => String::new(),
            (short, "") => short.to_string(),
            ("", long) => long.to_string(),
            (short, long) if short == long => short.to_string(),
            (short, long) => format!("{short}\n\n{long}"),
        }
    }
}

pub fn normalize_epg_text(value: &str) -> String {
    // Enigma2 often sends EPG strings with literal "\n" escapes and HTML entities.
    let value = value
        .replace("\\r\\n", "\n")
        .replace("\\n", "\n")
        .replace("\\r", "\n");
    decode_html_entities(&value)
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn deserialize_normalized_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.as_deref().map(normalize_epg_text).unwrap_or_default())
}

fn decode_html_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            output.push(ch);
            continue;
        }

        let mut entity = String::new();
        let mut terminated = false;
        while let Some(&next) = chars.peek() {
            chars.next();
            if next == ';' {
                terminated = true;
                break;
            }
            entity.push(next);
            if entity.len() > 12 {
                break;
            }
        }

        if terminated {
            if let Some(decoded) = decode_entity(&entity) {
                output.push(decoded);
                continue;
            }
        }

        output.push('&');
        output.push_str(&entity);
        if terminated {
            output.push(';');
        }
    }

    output
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "apos" => Some('\''),
        "auml" => Some('\u{00e4}'),
        "Auml" => Some('\u{00c4}'),
        "gt" => Some('>'),
        "lt" => Some('<'),
        "nbsp" => Some(' '),
        "ouml" => Some('\u{00f6}'),
        "Ouml" => Some('\u{00d6}'),
        "quot" => Some('"'),
        "szlig" => Some('\u{00df}'),
        "uuml" => Some('\u{00fc}'),
        "Uuml" => Some('\u{00dc}'),
        value if value.starts_with("#x") || value.starts_with("#X") => {
            u32::from_str_radix(&value[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        value if value.starts_with('#') => value[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

pub fn attach_epg(mut bouquet: Bouquet, events: &[EpgEvent]) -> Bouquet {
    let by_ref: HashMap<&str, &EpgEvent> = events
        .iter()
        .filter(|event| !event.sref.is_empty())
        .map(|event| (event.sref.as_str(), event))
        .collect();

    for channel in &mut bouquet.channels {
        channel.epg = by_ref
            .get(channel.service_ref.as_str())
            .map(|event| (*event).clone());
    }

    bouquet
}

pub fn format_time(timestamp: i64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    if timestamp <= 0 {
        return "--:--".to_string();
    }

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
    let local = chrono_like_local_time(datetime);
    format!("{:02}:{:02}", local.0, local.1)
}

fn chrono_like_local_time(time: std::time::SystemTime) -> (u64, u64) {
    let seconds = time
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let local_seconds = seconds + local_offset_seconds();
    ((local_seconds / 3600) % 24, (local_seconds / 60) % 60)
}

fn local_offset_seconds() -> u64 {
    // The player is used locally and EPG timestamps from Enigma2 are Unix seconds.
    // Keeping this small avoids pulling a full timezone dependency for UI labels.
    let offset = std::env::var("TZ_OFFSET_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(2 * 3600);
    offset.max(0) as u64
}
