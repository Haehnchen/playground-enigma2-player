use super::model::{attach_epg, Bouquet, EpgEvent, EpgNowResponse, ServicesResponse};
use super::urls::{epg_now_url, epg_service_url, extract_stream_url, services_url, stream_m3u_url};
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
            Self::MissingSettings(message)
            | Self::Http(message)
            | Self::Json(message)
            | Self::InvalidResponse(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Enigma2Error {}

#[derive(Clone)]
struct Cached<T> {
    value: T,
    fetched_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EpgEndpoint {
    Now,
    Service,
}

impl EpgEndpoint {
    fn url(self, base_url: &str, reference: &str) -> String {
        match self {
            Self::Now => epg_now_url(base_url, reference),
            Self::Service => epg_service_url(base_url, reference),
        }
    }
}

#[derive(Default)]
struct ClientState {
    base_url: String,
    services_cache: Option<Cached<Vec<Bouquet>>>,
    epg_cache: HashMap<(EpgEndpoint, String), Cached<Vec<EpgEvent>>>,
}

impl ClientState {
    fn clear_cache(&mut self) {
        self.services_cache = None;
        self.epg_cache.clear();
    }

    fn configured_base_url(&self) -> Result<String, Enigma2Error> {
        if self.base_url.is_empty() {
            return Err(Enigma2Error::MissingSettings(
                "Dreambox URL is not configured in settings".to_string(),
            ));
        }
        Ok(self.base_url.clone())
    }
}

#[derive(Clone)]
pub struct Enigma2Client {
    state: Arc<Mutex<ClientState>>,
    http_get: HttpGetter,
}

impl Enigma2Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_http_getter(base_url, http_get)
    }

    fn with_http_getter(
        base_url: impl Into<String>,
        http_get: impl Fn(&str) -> Result<String, Enigma2Error> + Send + Sync + 'static,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ClientState {
                base_url: super::urls::normalize_base_url(&base_url.into()),
                ..ClientState::default()
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
            state.clear_cache();
        }
    }

    pub fn clear_cache(&self) {
        self.state.lock().unwrap().clear_cache();
    }

    pub fn bouquets(&self) -> Result<Vec<Bouquet>, Enigma2Error> {
        let base_url = {
            let state = self.state.lock().unwrap();
            if let Some(cache) = &state.services_cache {
                if cache.fetched_at.elapsed() < SERVICES_CACHE_TTL {
                    return Ok(cache.value.clone());
                }
            }

            state.configured_base_url()?
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
        self.epg(EpgEndpoint::Now, bouquet_ref)
    }

    pub fn service_epg(&self, service_ref: &str) -> Result<Vec<EpgEvent>, Enigma2Error> {
        self.epg(EpgEndpoint::Service, service_ref)
    }

    fn epg(&self, endpoint: EpgEndpoint, reference: &str) -> Result<Vec<EpgEvent>, Enigma2Error> {
        let key = (endpoint, reference.to_string());
        let base_url = {
            let state = self.state.lock().unwrap();
            if let Some(cache) = state.epg_cache.get(&key) {
                if cache.fetched_at.elapsed() < EPG_CACHE_TTL {
                    return Ok(cache.value.clone());
                }
            }

            state.configured_base_url()?
        };

        let body = (self.http_get)(&endpoint.url(&base_url, reference))?;
        let parsed: EpgNowResponse =
            serde_json::from_str(&body).map_err(|err| Enigma2Error::Json(err.to_string()))?;
        let mut state = self.state.lock().unwrap();
        if state.base_url == base_url {
            state.epg_cache.insert(
                key,
                Cached {
                    value: parsed.events.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        Ok(parsed.events)
    }

    pub fn resolve_stream_url(&self, service_ref: &str) -> Result<String, Enigma2Error> {
        let base_url = self.state.lock().unwrap().configured_base_url()?;
        let body = (self.http_get)(&stream_m3u_url(&base_url, service_ref))?;
        extract_stream_url(&body).ok_or_else(|| {
            Enigma2Error::InvalidResponse("Dreambox stream playlist did not contain a URL".into())
        })
    }
}

fn http_get(url: &str) -> Result<String, Enigma2Error> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build()
        .into();
    let mut response = agent
        .get(url)
        .call()
        .map_err(|err| Enigma2Error::Http(format!("{url}: {err}")))?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|err| Enigma2Error::Http(format!("{url}: {err}")))
}

#[cfg(test)]
#[path = "../../tests/enigma2/api.rs"]
mod tests;
