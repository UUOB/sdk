//! The SDK for Kotlin and Swift (UniFFI): one `UuobClient` per app. Records,
//! lists and answers are JSON text, as in UUOB's own API, so the screens
//! read them with the JSON tools of their platform.
//!
//! ```kotlin
//! val uuob = UuobClient("photos", "Photos", packageName, filesDir.path)
//! when (uuob.check()) { "connected" -> …; "disconnected" -> …; else -> … }
//! val all = JSONObject(uuob.all("photos"))   // { records: [...], complete }
//! while (true) { val e = JSONObject(uuob.nextEvent()); … }
//! ```

use std::sync::Arc;

use serde_json::{Value as Json, json};
use tokio::sync::{Mutex, broadcast};

use crate::{App, Client, Error, Event, Result};

fn parse(s: &str) -> Json {
    if s.trim().is_empty() { Json::Null } else { serde_json::from_str(s).unwrap_or(Json::Null) }
}

fn opt(s: &Option<String>) -> Option<&str> {
    s.as_deref().filter(|s| !s.is_empty())
}

#[derive(uniffi::Object)]
pub struct UuobClient {
    client: Client,
    events: Mutex<broadcast::Receiver<Event>>,
}

impl UuobClient {
    /// The SDK itself, for an app's own Rust core that is handed this object.
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

#[uniffi::export]
impl UuobClient {
    /// `id`, `name`: who the app is. `android_package`: the app's package on
    /// Android (null elsewhere). `data_dir`: the app's private folder.
    #[uniffi::constructor]
    pub fn new(id: String, name: String, android_package: Option<String>, data_dir: String) -> Arc<Self> {
        let client = Client::new(App { id, name, android_package }, data_dir);
        let events = Mutex::new(client.subscribe());
        Arc::new(UuobClient { client, events })
    }

    pub fn has_token(&self) -> bool {
        self.client.has_token()
    }

    pub fn device(&self) -> Option<String> {
        self.client.device()
    }

    pub fn remote(&self) -> bool {
        self.client.remote()
    }

    pub fn pending(&self) -> u32 {
        self.client.pending() as u32
    }

    /// `connected`, `disconnected` or `unreachable`.
    pub async fn check(&self) -> String {
        self.client.check().await.as_str().into()
    }

    /// Empty or null `code`: the user is asked in UUOB.
    pub async fn connect(&self, code: Option<String>) -> Result<()> {
        self.client.connect(code).await
    }

    pub fn cancel_connect(&self) {
        self.client.cancel_connect()
    }

    /// Call when the app comes back to the front.
    pub fn resume(&self) {
        self.client.resume()
    }

    /// Waits for the next change: `{"type":"changed","container":"…"|"*"}`,
    /// `{"type":"state","state":"connected"|"disconnected"|"unreachable"}`,
    /// `{"type":"asking","asking":true|false}` or `{"type":"settings","settings":{…}}`.
    pub async fn next_event(&self) -> String {
        let mut rx = self.events.lock().await;
        let e = loop {
            match rx.recv().await {
                Ok(e) => break e,
                Err(broadcast::error::RecvError::Lagged(_)) => break Event::Changed("*".into()),
                Err(broadcast::error::RecvError::Closed) => std::future::pending().await,
            }
        };
        match e {
            Event::Changed(c) => json!({ "type": "changed", "container": c }),
            Event::State(s) => json!({ "type": "state", "state": s.as_str() }),
            Event::Asking(a) => json!({ "type": "asking", "asking": a }),
            Event::Settings(s) => json!({ "type": "settings", "settings": s }),
        }
        .to_string()
    }

    /// Any of UUOB's methods, as this app.
    pub async fn call(&self, method: String, params: String) -> Result<String> {
        self.client.call(&method, parse(&params)).await.map(|v| v.to_string())
    }

    pub async fn put(&self, collection: String, container: Option<String>, id: Option<String>, value: String) -> Result<String> {
        self.client.put(&collection, opt(&container), opt(&id), parse(&value)).await.map(|v| v.to_string())
    }

    pub async fn get(&self, collection: String, container: Option<String>, id: String) -> Result<String> {
        self.client.get(&collection, opt(&container), &id).await.map(|v| v.to_string())
    }

    pub async fn delete(&self, collection: String, container: Option<String>, id: String) -> Result<String> {
        self.client.delete(&collection, opt(&container), &id).await.map(|v| v.to_string())
    }

    /// The records, or while UUOB is away what the app last saw.
    pub async fn load(&self, collection: String, container: Option<String>) -> Result<String> {
        self.client.load(&collection, opt(&container)).await.map(|v| Json::Array(v).to_string())
    }

    /// What the app last saw, or null.
    pub fn cached(&self, collection: String, container: Option<String>) -> Option<String> {
        self.client.cached(&collection, opt(&container)).map(|v| Json::Array(v).to_string())
    }

    /// `{ records: [... each with its container], complete }`.
    pub async fn all(&self, collection: String) -> String {
        let (records, complete) = self.client.all(&collection).await;
        json!({ "records": records, "complete": complete }).to_string()
    }

    pub fn cached_all(&self, collection: String) -> String {
        Json::Array(self.client.cached_all(&collection)).to_string()
    }

    pub async fn containers(&self) -> Result<String> {
        self.client.containers().await.map(|v| Json::Array(v).to_string())
    }

    pub fn cached_containers(&self) -> Option<String> {
        self.client.cached_containers().map(|v| Json::Array(v).to_string())
    }

    /// Returns the new container's id (JSON string).
    pub async fn create_container(&self, name: String, key: Option<String>) -> Result<String> {
        self.client.create_container(&name, opt(&key)).await.map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))
    }

    pub async fn rename_container(&self, id: String, name: String) -> Result<()> {
        self.client.rename_container(&id, &name).await.map(drop)
    }

    pub async fn delete_container(&self, id: String) -> Result<()> {
        self.client.delete_container(&id).await.map(drop)
    }

    pub async fn keep(&self, id: String, keep: String) -> Result<()> {
        self.client.keep(&id, &keep).await.map(drop)
    }

    pub async fn share(&self, id: String) -> Result<()> {
        self.client.share(&id).await.map(drop)
    }

    pub async fn request_access(&self, container: String, access: String) -> Result<()> {
        self.client.request_access(&container, &access).await.map(drop)
    }

    /// `{ id, size, type, … }`.
    pub async fn put_file(&self, container: Option<String>, bytes: Vec<u8>, content_type: String) -> Result<String> {
        self.client.put_file(opt(&container), bytes, &content_type).await.map(|v| v.to_string())
    }

    /// The same, from a file on this device.
    pub async fn put_file_path(&self, container: Option<String>, path: String, content_type: String) -> Result<String> {
        self.client.put_file_path(opt(&container), path, &content_type).await.map(|v| v.to_string())
    }

    pub async fn file_url(&self, container: Option<String>, id: String) -> String {
        self.client.file_url(opt(&container), &id).await
    }

    pub async fn read_file(&self, container: Option<String>, id: String) -> Result<Vec<u8>> {
        self.client.read_file(opt(&container), &id).await
    }

    pub async fn delete_file(&self, container: Option<String>, id: String) -> Result<()> {
        self.client.delete_file(opt(&container), &id).await.map(drop)
    }

    pub fn cached_settings(&self) -> String {
        self.client.cached_settings().to_string()
    }

    pub async fn settings(&self) -> String {
        self.client.settings().await.to_string()
    }

    pub async fn set_settings(&self, change: String) -> String {
        self.client.set_settings(parse(&change)).await.to_string()
    }

    pub async fn upcoming(&self, list: String) {
        self.client.upcoming(parse(&list)).await
    }
}

/// A new record id.
#[uniffi::export]
pub fn new_record_id() -> String {
    crate::new_id()
}

/// The newer version of the app, if one is published at `feed`.
#[uniffi::export]
pub async fn update_check(feed: String, version: String) -> Result<Option<String>> {
    crate::on_runtime(async move { tokio::task::spawn_blocking(move || crate::update::check(&feed, &version)).await.expect("update check") })
        .await
        .map_err(|e| Error::new("update", e))
}

/// Downloads the newer version, checks it against `key` and installs it:
/// `{"restart": "<app>"}` on a Mac, `{"apk": "<file>"}` on Android (saved into
/// `dir` for the system installer), null when there is nothing newer.
#[uniffi::export]
pub async fn update_install(feed: String, key: String, version: String, dir: String) -> Result<String> {
    crate::on_runtime(async move {
        tokio::task::spawn_blocking(move || crate::update::install(&feed, &key, &version, &json!({ "dir": dir }))).await.expect("update install")
    })
    .await
    .map(|v| v.to_string())
    .map_err(|e| Error::new("update", e))
}

// ---- Dates (ui/dates.js) ---------------------------------------------------

/// Today on this device, "YYYY-MM-DD".
#[uniffi::export]
pub fn today() -> String {
    crate::dates::today()
}

#[uniffi::export]
pub fn add_days(date: String, n: i64) -> String {
    crate::dates::add_days(&date, n)
}

/// "Today", "Tomorrow", "Friday", "Mon 24 Nov" …
#[uniffi::export]
pub fn friendly_date(date: String) -> String {
    crate::dates::friendly(&date)
}

/// "14:30", or "2:30 PM" with `hour12`.
#[uniffi::export]
pub fn friendly_time(time: String, hour12: bool) -> String {
    crate::dates::friendly_time(&time, hour12)
}

/// A typed time ("1430", "9", "2pm") as "HH:MM", or null.
#[uniffi::export]
pub fn parse_time(text: String) -> Option<String> {
    crate::dates::parse_time(&text)
}

/// Quick-add text: `{"title","due","time"}` (due and time may be null).
#[uniffi::export]
pub fn parse_quick(text: String) -> String {
    serde_json::to_string(&crate::dates::parse_quick(&text)).unwrap_or_default()
}
