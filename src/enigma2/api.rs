use super::model::{attach_epg, Bouquet, EpgEvent, EpgNowResponse, ServicesResponse};
use super::urls::{epg_now_url, extract_stream_url, services_url, stream_m3u_url};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Short caches keep overlay navigation responsive without hiding receiver changes for long.
const SERVICES_CACHE_TTL: Duration = Duration::from_secs(60);
const EPG_CACHE_TTL: Duration = Duration::from_secs(90);
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

type HttpGetter = Arc<dyn Fn(&str) -> Result<String, Enigma2Error> + Send + Sync>;

#[derive(Debug)]
pub enum Enigma2Error {
    MissingSettings(String),
    Http(String),
    Json(String),
    InvalidResponse(String),
}

impl fmt::Display for Enigma2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSettings(message) => write!(f, "{message}"),
            Self::Http(message) => write!(f, "{message}"),
            Self::Json(message) => write!(f, "{message}"),
            Self::InvalidResponse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Enigma2Error {}

#[derive(Clone)]
struct Cached<T> {
    value: T,
    fetched_at: Instant,
}

struct ClientState {
    base_url: String,
    services_cache: Option<Cached<Vec<Bouquet>>>,
    epg_cache: HashMap<String, Cached<Vec<EpgEvent>>>,
}

#[derive(Clone)]
pub struct Enigma2Client {
    state: Arc<Mutex<ClientState>>,
    http_get: HttpGetter,
}

impl Enigma2Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ClientState {
                base_url: super::urls::normalize_base_url(&base_url.into()),
                services_cache: None,
                epg_cache: HashMap::new(),
            })),
            http_get: Arc::new(http_get),
        }
    }

    #[doc(hidden)]
    pub fn new_with_http_getter(
        base_url: impl Into<String>,
        http_get: impl Fn(&str) -> Result<String, Enigma2Error> + Send + Sync + 'static,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ClientState {
                base_url: super::urls::normalize_base_url(&base_url.into()),
                services_cache: None,
                epg_cache: HashMap::new(),
            })),
            http_get: Arc::new(http_get),
        }
    }

    pub fn base_url(&self) -> String {
        self.state.lock().unwrap().base_url.clone()
    }

    pub fn set_base_url(&self, base_url: impl Into<String>) {
        let normalized = super::urls::normalize_base_url(&base_url.into());
        let mut state = self.state.lock().unwrap();
        if state.base_url != normalized {
            state.base_url = normalized;
            state.services_cache = None;
            state.epg_cache.clear();
        }
    }

    pub fn clear_cache(&self) {
        let mut state = self.state.lock().unwrap();
        state.services_cache = None;
        state.epg_cache.clear();
    }

    fn configured_base_url(&self) -> Result<String, Enigma2Error> {
        let base_url = self.state.lock().unwrap().base_url.clone();
        if base_url.trim().is_empty() {
            return Err(Enigma2Error::MissingSettings(
                "Dreambox URL is not configured in settings".to_string(),
            ));
        }
        Ok(base_url)
    }

    pub fn bouquets(&self) -> Result<Vec<Bouquet>, Enigma2Error> {
        let base_url = {
            let state = self.state.lock().unwrap();
            if let Some(cache) = &state.services_cache {
                if cache.fetched_at.elapsed() < SERVICES_CACHE_TTL {
                    return Ok(cache.value.clone());
                }
            }

            if state.base_url.trim().is_empty() {
                return Err(Enigma2Error::MissingSettings(
                    "Dreambox URL is not configured in settings".to_string(),
                ));
            }
            state.base_url.clone()
        };

        let body = (self.http_get)(&services_url(&base_url))?;
        let parsed: ServicesResponse =
            serde_json::from_str(&body).map_err(|err| Enigma2Error::Json(err.to_string()))?;
        if !parsed.result {
            return Err(Enigma2Error::InvalidResponse(
                "Dreambox did not accept getallservices".to_string(),
            ));
        }

        let bouquets: Vec<Bouquet> = parsed
            .services
            .into_iter()
            .map(Bouquet::from)
            .filter(|bouquet| !bouquet.name.trim().is_empty() && !bouquet.channels.is_empty())
            .collect();
        let mut state = self.state.lock().unwrap();
        if state.base_url == base_url {
            state.services_cache = Some(Cached {
                value: bouquets.clone(),
                fetched_at: Instant::now(),
            });
        }
        Ok(bouquets)
    }

    pub fn bouquet_with_epg(&self, bouquet_index: usize) -> Result<Bouquet, Enigma2Error> {
        let bouquets = self.bouquets()?;
        let bouquet = bouquets
            .get(bouquet_index)
            .cloned()
            .ok_or_else(|| Enigma2Error::InvalidResponse("Bouquet not found".to_string()))?;
        let events = self.epg_now(&bouquet.service_ref)?;
        Ok(attach_epg(bouquet, &events))
    }

    pub fn epg_now(&self, bouquet_ref: &str) -> Result<Vec<EpgEvent>, Enigma2Error> {
        let base_url = {
            let state = self.state.lock().unwrap();
            if let Some(cache) = state.epg_cache.get(bouquet_ref) {
                if cache.fetched_at.elapsed() < EPG_CACHE_TTL {
                    return Ok(cache.value.clone());
                }
            }

            if state.base_url.trim().is_empty() {
                return Err(Enigma2Error::MissingSettings(
                    "Dreambox URL is not configured in settings".to_string(),
                ));
            }
            state.base_url.clone()
        };

        let body = (self.http_get)(&epg_now_url(&base_url, bouquet_ref))?;
        let parsed: EpgNowResponse =
            serde_json::from_str(&body).map_err(|err| Enigma2Error::Json(err.to_string()))?;
        let mut state = self.state.lock().unwrap();
        if state.base_url == base_url {
            state.epg_cache.insert(
                bouquet_ref.to_string(),
                Cached {
                    value: parsed.events.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        Ok(parsed.events)
    }

    pub fn resolve_stream_url(&self, service_ref: &str) -> Result<String, Enigma2Error> {
        let base_url = self.configured_base_url()?;
        let body = (self.http_get)(&stream_m3u_url(&base_url, service_ref))?;
        extract_stream_url(&body).ok_or_else(|| {
            Enigma2Error::InvalidResponse("Dreambox stream playlist did not contain a URL".into())
        })
    }
}

fn http_get(url: &str) -> Result<String, Enigma2Error> {
    let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();
    let response = agent
        .get(url)
        .call()
        .map_err(|err| Enigma2Error::Http(format!("{url}: {err}")))?;
    response
        .into_string()
        .map_err(|err| Enigma2Error::Http(format!("{url}: {err}")))
}
