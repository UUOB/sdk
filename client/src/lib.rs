//! The UUOB SDK for native apps. It is what `ui/uuob.js` is for a web UI, with
//! the same behaviour and the same data, so an app's native and web versions
//! can run side by side:
//!
//! ```ignore
//! let uuob = Client::new(App { id: "photos".into(), name: "Photos".into(), android_package: None }, dir);
//! uuob.check().await;                          // Connected / Disconnected / Unreachable
//! uuob.connect(None).await?;                   // the user taps Allow in UUOB
//! uuob.put("photos", Some(&month), None, json!({ "taken": 1 })).await?;
//! let (records, complete) = uuob.all("photos").await;
//! while let Ok(event) = events.recv().await { … }  // `uuob.subscribe()`
//! ```
//!
//! The app keeps no backend and no data of its own: its records live in its
//! containers in UUOB. While UUOB is unreachable the app keeps working on what
//! it last saw; changes to records wait on the device (only the fields that
//! changed) and are sent when UUOB is back.
//!
//! Kotlin and Swift use it through UniFFI ([`ffi`]), with JSON in and out.

pub mod dates;
pub mod ffi;
mod http;
mod store;
pub mod update;

uniffi::setup_scaffolding!();

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{Map, Value as Json, json};
use tokio::sync::{Notify, OnceCell, broadcast};
use uuob_core::{AppManifest, NetMode, Tunnel};

use store::{Change, Store, with_pending};

/// The runtime the SDK's work runs on, whoever awaits it (Kotlin, Swift, GTK).
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().thread_name("uuob-client").build().expect("runtime"))
}

async fn on_runtime<T: Send + 'static>(fut: impl Future<Output = T> + Send + 'static) -> T {
    runtime().spawn(fut).await.expect("UUOB client task failed")
}

/// Who the app is.
#[derive(Clone, Debug)]
pub struct App {
    pub id: String,
    pub name: String,
    /// On Android: the app's package, so UUOB on this phone hands it its
    /// alerts and widget data (and nobody else's).
    pub android_package: Option<String>,
}

/// Where the app stands with UUOB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Connected,
    /// UUOB is here, but this app isn't connected (or was disconnected).
    Disconnected,
    /// No UUOB to talk to right now.
    Unreachable,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Connected => "connected",
            State::Disconnected => "disconnected",
            State::Unreachable => "unreachable",
        }
    }
}

/// What happened, for the screens.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Data the app can see changed: in this container, or `"*"` (anything).
    Changed(String),
    State(State),
    /// A question waits for the user in UUOB on another device (`true`), or
    /// was answered (`false`).
    Asking(bool),
    /// The settings for all apps changed here (see [`Client::set_settings`]).
    Settings(Json),
}

/// A call that failed: `code` is `unreachable`, `unauthorized`,
/// `permission_denied`, `not_found`, `invalid`, `unavailable`, …
#[derive(Debug, Clone, thiserror::Error, uniffi::Error)]
pub enum UuobError {
    #[error("{detail}")]
    Failed { code: String, detail: String },
}

/// (Kotlin and Swift see it as `UuobException` / `UuobError`.)
pub type Error = UuobError;

impl UuobError {
    pub fn new(code: &str, message: impl Into<String>) -> Error {
        Error::Failed { code: code.into(), detail: message.into() }
    }
    pub fn code(&self) -> &str {
        let Error::Failed { code, .. } = self;
        code
    }
    fn unreachable() -> Error {
        Error::new("unreachable", "UUOB isn’t running on this device")
    }
    fn is_unreachable(&self) -> bool {
        self.code() == "unreachable"
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// The settings every app shares until the user changes them.
fn default_settings() -> Json {
    json!({ "hour12": false, "sunday_first": false })
}

/// The port of the UUOB app on this device, for this user (UUOB's own rule,
/// `uuob_core::local_port`), unless `UUOB_PORT` names another.
pub fn local_port() -> u16 {
    std::env::var("UUOB_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or_else(uuob_core::local_port)
}

/// Asks UUOB on this device, through the system, for its answer to a nonce
/// (Android: an ordered broadcast to app.uuob, which only the real UUOB
/// receives). Blocking. `None`: UUOB isn't installed, or its port isn't its own.
pub type Prover = Arc<dyn Fn(String) -> Option<String> + Send + Sync>;

/// A new record id, as `uuob.js` makes them: 16 characters of a–z, 2–7.
pub fn new_id() -> String {
    use rand::RngCore;
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| b"abcdefghijklmnopqrstuvwxyz234567"[(x % 32) as usize] as char).collect()
}

/// The app's connection to UUOB. Cheap to clone; make one per app.
#[derive(Clone)]
pub struct Client(Arc<Inner>);

struct Inner {
    app: App,
    dir: PathBuf,
    store: Store,
    token: Mutex<Option<String>>,
    port: Mutex<u16>,
    /// The port was found (the tunnel, when linked, or UUOB on this device).
    located: Mutex<Arc<OnceCell<()>>>,
    tunnel: OnceCell<Option<Tunnel>>,
    /// UUOB is on another device.
    remote: AtomicBool,
    device: Mutex<Option<String>>,
    events: broadcast::Sender<Event>,
    stream: Mutex<Option<tokio::task::AbortHandle>>,
    flushing: tokio::sync::Mutex<()>,
    cancel: Notify,
    upcoming: tokio::sync::Mutex<Option<String>>,
    settings_changes: AtomicU64,
    android_given: AtomicBool,
    /// Set on Android (see [`Client::set_prover`]).
    prover: Mutex<Option<Prover>>,
    /// The server on the port has proven it is UUOB (or there is no need to).
    trusted: AtomicBool,
}

impl Client {
    /// `dir`: the app's private data folder; the SDK keeps its files in a
    /// `uuob` folder inside it.
    pub fn new(app: App, dir: impl Into<PathBuf>) -> Client {
        let dir = dir.into().join("uuob");
        let store = Store::new(dir.clone());
        let token = store.token();
        Client(Arc::new(Inner {
            app,
            dir,
            store,
            token: Mutex::new(token),
            port: Mutex::new(local_port()),
            located: Mutex::new(Arc::new(OnceCell::new())),
            tunnel: OnceCell::new(),
            remote: AtomicBool::new(false),
            device: Mutex::new(None),
            events: broadcast::channel(256).0,
            stream: Mutex::new(None),
            flushing: tokio::sync::Mutex::new(()),
            cancel: Notify::new(),
            upcoming: tokio::sync::Mutex::new(None),
            settings_changes: AtomicU64::new(0),
            android_given: AtomicBool::new(false),
            prover: Mutex::new(None),
            trusted: AtomicBool::new(true),
        }))
    }

    pub fn app(&self) -> &App {
        &self.0.app
    }

    /// Changes, as they happen (see [`Event`]).
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.0.events.subscribe()
    }

    fn emit(&self, e: Event) {
        let _ = self.0.events.send(e);
    }

    /// Connected before (has a key)? Then the app can show cached data at once.
    pub fn has_token(&self) -> bool {
        self.0.token.lock().unwrap().is_some()
    }

    fn token(&self) -> Option<String> {
        self.0.token.lock().unwrap().clone()
    }

    /// The name of the device UUOB runs on, once [`Client::check`] has seen it.
    pub fn device(&self) -> Option<String> {
        self.0.device.lock().unwrap().clone()
    }

    /// UUOB is on another device (the app was linked with a code).
    pub fn remote(&self) -> bool {
        self.0.remote.load(Relaxed)
    }

    /// How many changes wait for UUOB to come back.
    pub fn pending(&self) -> usize {
        self.0.store.outbox().len()
    }

    // ---- Finding UUOB --------------------------------------------------

    async fn tunnel(&self) -> Option<Tunnel> {
        let dir = self.0.dir.clone();
        self.0.tunnel.get_or_init(|| async move { Tunnel::open(dir, NetMode::Internet).await.ok() }).await.clone()
    }

    /// Where UUOB's API is: the tunnel if the app is linked to a UUOB on
    /// another device, otherwise UUOB on this one.
    async fn port(&self) -> u16 {
        let located = self.0.located.lock().unwrap().clone();
        located
            .get_or_init(|| async {
                if let Some(t) = self.tunnel().await
                    && t.linked().await
                {
                    *self.0.port.lock().unwrap() = t.port();
                    self.0.remote.store(true, Relaxed);
                } else {
                    *self.0.port.lock().unwrap() = local_port();
                }
            })
            .await;
        *self.0.port.lock().unwrap()
    }

    fn relocate(&self) {
        *self.0.located.lock().unwrap() = Arc::new(OnceCell::new());
    }

    /// On Android: check with UUOB itself, through the system, before
    /// anything is sent to its port. Another app could take the port while
    /// UUOB is not running and collect what apps send there.
    pub fn set_prover(&self, prover: Prover) {
        *self.0.prover.lock().unwrap() = Some(prover);
        self.0.trusted.store(false, Relaxed);
    }

    /// The port, once the server on it has proven it is UUOB: it answers a
    /// random nonce as UUOB on this device answers it through the system (see
    /// `uuob_api::proof`). A tunnel to another device is the app's own.
    async fn verified_port(&self) -> Result<u16> {
        let port = self.port().await;
        let prover = self.0.prover.lock().unwrap().clone();
        let Some(prover) = prover.filter(|_| !self.0.trusted.load(Relaxed)) else { return Ok(port) };
        let nonce = new_id();
        let res = http::send(port, &format!("?prove={nonce}"), None, None).await.map_err(|_| Error::unreachable())?;
        let (_, bytes) = http::read(res).await.map_err(|_| Error::unreachable())?;
        let hello: Json = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        if !hello["remote"].as_bool().unwrap_or(false) {
            let mine = tokio::task::spawn_blocking(move || prover(nonce)).await.ok().flatten();
            if mine.is_none() || mine.as_deref() != hello["proof"].as_str() {
                return Err(Error::new("unreachable", "Another app is using UUOB’s port, so nothing was sent to it"));
            }
        }
        self.0.trusted.store(true, Relaxed);
        Ok(port)
    }

    /// UUOB went away: whatever answers on its port next proves itself again.
    fn lost(&self) -> Error {
        if self.0.prover.lock().unwrap().is_some() {
            self.0.trusted.store(false, Relaxed);
        }
        Error::unreachable()
    }

    // ---- Requests ------------------------------------------------------

    /// One request to UUOB's API (`path` after `/api`); `body` is JSON unless
    /// it comes with its own type (a file).
    async fn request(&self, path: &str, body: Option<(Vec<u8>, &str)>) -> Result<Json> {
        let port = self.verified_port().await?;
        let token = self.token();
        let res = http::send(port, path, token.as_deref(), body).await.map_err(|_| self.lost())?;
        let (status, bytes) = http::read(res).await.map_err(|_| self.lost())?;
        let data: Json = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        if status == 401 {
            self.forget();
            return Err(Error::new("unauthorized", "Not connected to UUOB"));
        }
        if let Some(e) = data.get("error") {
            let code = e["code"].as_str().unwrap_or("failed");
            return Err(Error::new(code, e["message"].as_str().unwrap_or(code)));
        }
        Ok(data)
    }

    async fn post(&self, path: &str, body: &Json) -> Result<Json> {
        self.request(path, Some((body.to_string().into_bytes(), "application/json"))).await
    }

    /// Runs one of UUOB's methods as this app (`put`, `list`, `containers`, …)
    /// and returns its result.
    pub async fn call(&self, method: &str, params: Json) -> Result<Json> {
        let c = self.clone();
        let method = method.to_string();
        on_runtime(async move {
            let r = c.post("/call", &json!({ "method": method, "params": params })).await;
            match r {
                Ok(mut data) => Ok(data["result"].take()),
                Err(e) => {
                    if e.is_unreachable() {
                        c.emit(Event::State(State::Unreachable));
                    }
                    Err(e)
                }
            }
        })
        .await
    }

    /// The user's answer is needed in UUOB. On another device the app can't
    /// bring it to the front, so the screens tell the user to go there.
    async fn asking(&self, method: &str, params: Json) -> Result<Json> {
        let remote = self.remote();
        if remote {
            self.emit(Event::Asking(true));
        }
        let r = self.call(method, params).await;
        if remote {
            self.emit(Event::Asking(false));
        }
        r
    }

    fn forget(&self) {
        *self.0.token.lock().unwrap() = None;
        self.0.store.set_token(None);
        self.0.store.clear_cache();
        self.0.store.clear_outbox();
        if let Some(s) = self.0.stream.lock().unwrap().take() {
            s.abort();
        }
        self.0.android_given.store(false, Relaxed);
        self.emit(Event::State(State::Disconnected));
    }

    // ---- Connecting ----------------------------------------------------

    /// Is UUOB there, and is this app connected to it?
    pub async fn check(&self) -> State {
        let c = self.clone();
        on_runtime(async move { c.check_now().await }).await
    }

    async fn check_now(&self) -> State {
        let hello = match self.request("", None).await {
            Ok(h) => h,
            Err(_) => {
                // Linked to a UUOB on another device, where the user
                // disconnected this app? Then that link is gone.
                if self.has_token()
                    && let Some(t) = self.tunnel().await
                    && t.removed()
                {
                    self.forget();
                    self.0.remote.store(false, Relaxed);
                    self.relocate();
                }
                return State::Unreachable;
            }
        };
        *self.0.device.lock().unwrap() = hello["device"].as_str().map(Into::into);
        let remote = hello["remote"].as_bool().unwrap_or(false);
        self.0.remote.store(remote, Relaxed);
        if !self.has_token() {
            return if remote { State::Unreachable } else { State::Disconnected };
        }
        if let Err(e) = self.post("/call", &json!({ "method": "containers" })).await {
            return if e.is_unreachable() { State::Unreachable } else { State::Disconnected };
        }
        if self.0.stream.lock().unwrap().is_none() {
            self.follow();
        }
        self.register_android().await;
        State::Connected
    }

    /// Connect to UUOB. Without a code the user is asked in UUOB (this waits
    /// for their answer, or [`Client::cancel_connect`]). With a code from UUOB →
    /// Apps it connects at once; if this device has no UUOB, the code links
    /// the app to the UUOB on the device that showed it.
    pub async fn connect(&self, code: Option<String>) -> Result<()> {
        let c = self.clone();
        on_runtime(async move { c.connect_now(code.filter(|c| !c.trim().is_empty())).await }).await
    }

    async fn connect_now(&self, code: Option<String>) -> Result<()> {
        let here = match &code {
            Some(_) => self.request("", None).await.is_ok_and(|h| !h["remote"].as_bool().unwrap_or(false)),
            None => false,
        };
        let token = if let (Some(code), false) = (&code, here) {
            let t = self.tunnel().await.ok_or_else(|| Error::new("pairing", "Couldn’t start the connection to your other device."))?;
            let manifest = AppManifest { id: self.0.app.id.clone(), name: self.0.app.name.clone() };
            let token = t.link(code, &manifest).await.map_err(|e| {
                let text = e.to_string();
                Error::new("pairing", text.split_once(": ").map_or(text.clone(), |(_, m)| m.to_string()))
            })?;
            *self.0.port.lock().unwrap() = t.port();
            self.0.remote.store(true, Relaxed);
            token
        } else {
            let mut body = json!({ "app": self.0.app.id, "name": self.0.app.name });
            if let Some(code) = &code {
                body["code"] = json!(code);
            }
            let r = tokio::select! {
                r = self.post("/connect", &body) => r?,
                _ = self.0.cancel.notified() => return Err(Error::new("cancelled", "Cancelled")),
            };
            r["token"].as_str().ok_or_else(|| Error::new("invalid", "UUOB gave no key"))?.to_string()
        };
        *self.0.token.lock().unwrap() = Some(token.clone());
        self.0.store.set_token(Some(&token));
        self.follow();
        self.emit(Event::State(State::Connected));
        self.register_android().await;
        Ok(())
    }

    /// Stops waiting for the user's answer to [`Client::connect`].
    pub fn cancel_connect(&self) {
        self.0.cancel.notify_waiters();
    }

    /// On Android: tells UUOB on this phone which package this app is.
    async fn register_android(&self) {
        let Some(package) = self.0.app.android_package.clone() else { return };
        if self.remote() || self.0.android_given.load(Relaxed) {
            return;
        }
        if self.post("/call", &json!({ "method": "android", "params": { "package": package } })).await.is_ok() {
            self.0.android_given.store(true, Relaxed);
        }
    }

    // ---- Changes, live -------------------------------------------------

    /// Follows UUOB's server-sent events until the app is disconnected;
    /// retries every few seconds while UUOB is away.
    fn follow(&self) {
        let c = self.clone();
        let task = runtime().spawn(async move {
            loop {
                let token = c.token();
                let sent = match c.verified_port().await {
                    Ok(port) => http::send(port, "/events", token.as_deref(), None).await.map_err(|_| c.lost()).ok(),
                    Err(_) => None,
                };
                if let Some(res) = sent {
                    if res.status() == 401 {
                        c.forget();
                        return;
                    }
                    if res.status().is_success() {
                        c.emit(Event::State(State::Connected));
                        let flush = c.clone();
                        tokio::spawn(async move { flush.flush().await });
                        let mut body = res.into_body();
                        let mut buf = String::new();
                        while let Some(chunk) = http::chunk(&mut body).await {
                            buf.push_str(&String::from_utf8_lossy(&chunk));
                            while let Some(i) = buf.find("\n\n") {
                                let block: String = buf.drain(..i + 2).collect();
                                if let Some(line) = block.lines().find(|l| l.starts_with("data:"))
                                    && let Ok(v) = serde_json::from_str::<Json>(&line[5..])
                                    && let Some(container) = v["container"].as_str()
                                {
                                    c.emit(Event::Changed(container.into()));
                                }
                            }
                        }
                    }
                }
                // UUOB went away: try again until it's back.
                c.emit(Event::State(State::Unreachable));
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
        if let Some(old) = self.0.stream.lock().unwrap().replace(task.abort_handle()) {
            old.abort();
        }
    }

    /// The app came back to the front: a phone may have frozen it and the
    /// stream may be dead without an error. Starts a fresh one, which also
    /// reloads everything that changed meanwhile.
    pub fn resume(&self) {
        if self.has_token() && self.0.stream.lock().unwrap().is_some() {
            let c = self.clone();
            let _g = runtime().enter();
            c.follow();
        }
    }

    // ---- Records -------------------------------------------------------

    fn own(&self, container: Option<&str>) -> String {
        container.unwrap_or(&self.0.app.id).to_string()
    }

    fn known(&self, container: &str, collection: &str) -> Vec<Json> {
        match self.0.store.cached(container, collection) {
            Some(Json::Array(a)) => a,
            _ => vec![],
        }
    }

    fn base(collection: &str, container: Option<&str>) -> Json {
        let mut b = json!({ "collection": collection });
        if let Some(c) = container {
            b["container"] = json!(c);
        }
        b
    }

    /// UUOB is away: keep what the user did to a record (`value` None: removed
    /// it) and show it at once. Only the fields that differ from what the app
    /// last saw are kept, so a change made meanwhile on another device to
    /// another field of the same record survives.
    fn later(&self, container: &str, collection: &str, id: &str, value: Option<&Json>) -> Json {
        let records = self.known(container, collection);
        let before = records.iter().find(|r| r["id"] == json!(id)).and_then(|r| r.as_object().cloned());
        let entry = match value {
            None => Change { container: container.into(), collection: collection.into(), id: id.into(), remove: true, fresh: false, set: Map::new(), unset: vec![] },
            Some(v) => {
                let mut fields = v.as_object().cloned().unwrap_or_default();
                fields.remove("id");
                let set = fields.iter().filter(|(k, v)| before.as_ref().is_none_or(|b| b.get(*k) != Some(*v))).map(|(k, v)| (k.clone(), v.clone())).collect();
                let unset = before.as_ref().map(|b| b.keys().filter(|k| *k != "id" && !fields.contains_key(*k)).cloned().collect()).unwrap_or_default();
                Change { container: container.into(), collection: collection.into(), id: id.into(), remove: false, fresh: before.is_none(), set, unset }
            }
        };
        self.0.store.push(entry);
        let now = with_pending(&self.0.store.outbox(), container, collection, records);
        // Applying a waiting change again later is harmless: it sets the same fields.
        self.0.store.cache(container, collection, &Json::Array(now.clone()));
        let c = self.clone();
        runtime().spawn(async move { c.flush().await });
        match value {
            Some(_) => now.into_iter().find(|r| r["id"] == json!(id)).unwrap_or(Json::Null),
            None => json!(before.is_some()),
        }
    }

    /// Sends what waited. A change to a record that was deleted meanwhile is
    /// dropped, as is one UUOB refuses; on any other error the rest waits.
    async fn flush(&self) {
        let Ok(_g) = self.0.flushing.try_lock() else { return };
        if self.0.store.outbox().is_empty() {
            return;
        }
        while let Some(e) = self.0.store.outbox().into_iter().next() {
            let at = json!({ "container": e.container, "collection": e.collection, "id": e.id });
            let r = if e.remove {
                self.call("delete", at).await.map(drop)
            } else {
                match self.call("get", at.clone()).await {
                    Ok(now) if !now.is_null() || e.fresh => {
                        let mut value = now.as_object().cloned().unwrap_or_default();
                        for (k, v) in &e.set {
                            value.insert(k.clone(), v.clone());
                        }
                        value.remove("id");
                        for k in &e.unset {
                            value.remove(k);
                        }
                        let mut p = at.clone();
                        p["value"] = Json::Object(value);
                        self.call("put", p).await.map(drop)
                    }
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            };
            // Only a refusal drops a change; anything else (UUOB away, a full
            // disk, …) keeps it waiting for the next try.
            if let Err(e) = r
                && !matches!(e.code(), "permission_denied" | "not_found" | "invalid")
            {
                return;
            }
            self.0.store.shift();
        }
        self.emit(Event::Changed("*".into()));
    }

    /// While changes wait (or UUOB is away), a new change waits too.
    async fn send(&self, method: &str, params: Json) -> Result<Json> {
        if !self.0.store.outbox().is_empty() {
            return Err(Error::unreachable());
        }
        self.call(method, params).await
    }

    /// Writes a record to `collection` in this app's container, or in
    /// `container`; `id` (or `value.id`) or a new one. Returns the record.
    pub async fn put(&self, collection: &str, container: Option<&str>, id: Option<&str>, value: Json) -> Result<Json> {
        let own = self.own(container);
        let id = id.map(String::from).or_else(|| value["id"].as_str().map(String::from));
        let mut p = Self::base(collection, container);
        if let Some(id) = &id {
            p["id"] = json!(id);
        }
        p["value"] = value.clone();
        let r = match self.send("put", p).await {
            Ok(r) => r,
            Err(e) if e.is_unreachable() => self.later(&own, collection, &id.unwrap_or_else(new_id), Some(&value)),
            Err(e) => return Err(e),
        };
        self.emit(Event::Changed(own));
        Ok(r)
    }

    /// One record, or null. While UUOB is away: what the app last saw.
    pub async fn get(&self, collection: &str, container: Option<&str>, id: &str) -> Result<Json> {
        let mut p = Self::base(collection, container);
        p["id"] = json!(id);
        match self.call("get", p).await {
            Err(e) if e.is_unreachable() => {
                let own = self.own(container);
                let all = with_pending(&self.0.store.outbox(), &own, collection, self.known(&own, collection));
                Ok(all.into_iter().find(|r| r["id"] == json!(id)).unwrap_or(Json::Null))
            }
            r => r,
        }
    }

    /// Deletes a record; true if there was one.
    pub async fn delete(&self, collection: &str, container: Option<&str>, id: &str) -> Result<Json> {
        let own = self.own(container);
        let mut p = Self::base(collection, container);
        p["id"] = json!(id);
        let r = match self.send("delete", p).await {
            Ok(r) => r,
            Err(e) if e.is_unreachable() => self.later(&own, collection, id, None),
            Err(e) => return Err(e),
        };
        self.emit(Event::Changed(own));
        Ok(r)
    }

    /// Every record of `collection` in the container, from UUOB (and kept for
    /// later).
    pub async fn list(&self, collection: &str, container: Option<&str>) -> Result<Vec<Json>> {
        let own = self.own(container);
        let r = self.call("list", Self::base(collection, container)).await?;
        let records = match r {
            Json::Array(a) => a,
            _ => vec![],
        };
        self.0.store.cache(&own, collection, &Json::Array(records.clone()));
        Ok(with_pending(&self.0.store.outbox(), &own, collection, records))
    }

    /// What the app last saw of a collection, with the waiting changes: for
    /// showing at once. None if it never saw it.
    pub fn cached(&self, collection: &str, container: Option<&str>) -> Option<Vec<Json>> {
        let own = self.own(container);
        let Some(Json::Array(a)) = self.0.store.cached(&own, collection) else { return None };
        Some(with_pending(&self.0.store.outbox(), &own, collection, a))
    }

    /// [`Client::list`], or while UUOB is away what the app last saw.
    pub async fn load(&self, collection: &str, container: Option<&str>) -> Result<Vec<Json>> {
        match self.list(collection, container).await {
            Err(e) if e.is_unreachable() => self.cached(collection, container).ok_or(e),
            r => r,
        }
    }

    /// Every record of `collection` across all of this app's containers
    /// (including ones shared with the user), each with the id of its
    /// `container`, and whether every container delivered its records.
    pub async fn all(&self, collection: &str) -> (Vec<Json>, bool) {
        let containers = self.containers().await.or_else(|_| self.cached_containers().ok_or(())).unwrap_or_default();
        let mut tasks = vec![];
        for c in &containers {
            let id = c["id"].as_str().unwrap_or_default().to_string();
            let me = self.clone();
            let collection = collection.to_string();
            tasks.push(runtime().spawn(async move {
                let r = me.load(&collection, Some(&id)).await;
                (id, r)
            }));
        }
        let mut out = vec![];
        let mut complete = true;
        for t in tasks {
            let Ok((id, r)) = t.await else { continue };
            match r {
                Ok(records) => out.extend(records.into_iter().map(|r| with_container(r, &id))),
                Err(_) => complete = false,
            }
        }
        (out, complete)
    }

    /// [`Client::all`] from what the app last saw, for showing at once.
    pub fn cached_all(&self, collection: &str) -> Vec<Json> {
        let containers = self.cached_containers().unwrap_or_default();
        containers
            .iter()
            .filter_map(|c| c["id"].as_str())
            .flat_map(|id| self.cached(collection, Some(id)).unwrap_or_default().into_iter().map(move |r| with_container(r, id)))
            .collect()
    }

    // ---- Containers ----------------------------------------------------

    /// This app's containers: its first, private one (`main`) plus one per
    /// list, calendar, month … including those shared with the user. Each is
    /// `{ id, name, main, canWrite, from, sharedWith, keep, here }`.
    pub async fn containers(&self) -> Result<Vec<Json>> {
        let all = self.call("containers", json!({})).await?;
        let mine: Vec<Json> = all
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["owner_app"] == json!(self.0.app.id))
            .map(|c| {
                json!({
                    "id": c["id"], "name": c["name"], "main": c["id"] == json!(self.0.app.id),
                    "canWrite": c["can_write"], "from": c["from"], "sharedWith": c["shared_with"],
                    "keep": c["keep"], "here": c["here"],
                })
            })
            .collect();
        self.0.store.cache("", "containers", &Json::Array(mine.clone()));
        Ok(mine)
    }

    pub fn cached_containers(&self) -> Option<Vec<Json>> {
        match self.0.store.cached("", "containers") {
            Some(Json::Array(a)) => Some(a),
            _ => None,
        }
    }

    fn changed_all<T>(&self, r: T) -> T {
        self.emit(Event::Changed("*".into()));
        r
    }

    /// A new container; returns its id. `key`: reuse an existing random id,
    /// so that the same thing started on two devices is one container.
    pub async fn create_container(&self, name: &str, key: Option<&str>) -> Result<Json> {
        let mut p = json!({ "name": name });
        if let Some(k) = key {
            p["key"] = json!(k);
        }
        self.call("create_container", p).await.map(|r| self.changed_all(r))
    }

    pub async fn rename_container(&self, id: &str, name: &str) -> Result<Json> {
        self.call("rename_container", json!({ "id": id, "name": name })).await.map(|r| self.changed_all(r))
    }

    /// Deletes it everywhere; for one shared with the user, leaves it.
    pub async fn delete_container(&self, id: &str) -> Result<Json> {
        self.call("delete_container", json!({ "id": id })).await.map(|r| self.changed_all(r))
    }

    /// How much of it this device keeps: `all`, `list` (records; files come
    /// when opened) or `none`.
    pub async fn keep(&self, id: &str, keep: &str) -> Result<Json> {
        self.call("keep", json!({ "container": id, "keep": keep })).await.map(|r| self.changed_all(r))
    }

    /// The user chooses in UUOB who to share it with. (On Android the app
    /// first brings UUOB to the front, `uuob://share`.)
    pub async fn share(&self, id: &str) -> Result<Json> {
        self.asking("share", json!({ "container": id })).await
    }

    /// Asks the user for access to another app's data (`read` or `write`).
    pub async fn request_access(&self, container: &str, access: &str) -> Result<Json> {
        self.asking("request_access", json!({ "container": container, "access": access })).await
    }

    // ---- Files ---------------------------------------------------------

    /// Stores a file in a container: `{ id, size, type, … }`. Keep the id in
    /// a record. Needs UUOB (files don't wait).
    pub async fn put_file(&self, container: Option<&str>, bytes: Vec<u8>, content_type: &str) -> Result<Json> {
        let c = self.clone();
        let own = self.own(container);
        let ty = if content_type.is_empty() { "application/octet-stream".to_string() } else { content_type.to_string() };
        let path = format!("/files?container={}", encode(&own));
        let r = on_runtime(async move { c.request(&path, Some((bytes, &ty))).await }).await;
        if r.as_ref().is_err_and(|e| e.is_unreachable()) {
            self.emit(Event::State(State::Unreachable));
        }
        let r = r?;
        self.emit(Event::Changed(own));
        Ok(r)
    }

    /// [`Client::put_file`] from a file on this device (a big video doesn't
    /// pass through the app's own memory).
    pub async fn put_file_path(&self, container: Option<&str>, path: impl Into<PathBuf>, content_type: &str) -> Result<Json> {
        let path = path.into();
        let bytes = on_runtime(async move { tokio::fs::read(&path).await }).await.map_err(|e| Error::new("invalid", format!("The file can’t be read ({e}).")))?;
        self.put_file(container, bytes, content_type).await
    }

    /// Where a file can be read (by an image or video view, or any HTTP
    /// client; ranges work).
    pub async fn file_url(&self, container: Option<&str>, id: &str) -> String {
        let c = self.clone();
        // The address carries the app's key: none until the port is proven.
        let Ok(port) = on_runtime(async move { c.verified_port().await }).await else { return String::new() };
        let own = self.own(container);
        format!("http://127.0.0.1:{port}/api/files/{}?container={}&key={}", encode(id), encode(&own), self.token().unwrap_or_default())
    }

    /// A file's bytes.
    pub async fn read_file(&self, container: Option<&str>, id: &str) -> Result<Vec<u8>> {
        let c = self.clone();
        let path = format!("/files/{}?container={}&key={}", encode(id), encode(&self.own(container)), self.token().unwrap_or_default());
        on_runtime(async move {
            let port = c.verified_port().await?;
            let res = http::send(port, &path, None, None).await.map_err(|_| c.lost())?;
            let (status, bytes) = http::read(res).await.map_err(|_| c.lost())?;
            if status >= 400 {
                let data: Json = serde_json::from_slice(&bytes).unwrap_or(json!({}));
                let code = data["error"]["code"].as_str().unwrap_or("unavailable");
                return Err(Error::new(code, data["error"]["message"].as_str().unwrap_or("The file can’t be read")));
            }
            Ok(bytes)
        })
        .await
    }

    /// `[{ id, size, type, added, here }]`.
    pub async fn files(&self, container: Option<&str>) -> Result<Json> {
        self.call("files", json!({ "container": self.own(container) })).await
    }

    pub async fn delete_file(&self, container: Option<&str>, id: &str) -> Result<Json> {
        let own = self.own(container);
        let r = self.call("delete_file", json!({ "container": own, "id": id })).await?;
        self.emit(Event::Changed(own));
        Ok(r)
    }

    // ---- Settings for all apps -----------------------------------------

    /// `{ hour12, sunday_first }`, as last known (for showing at once).
    pub fn cached_settings(&self) -> Json {
        let mut s = default_settings();
        if let Some(Json::Object(c)) = self.0.store.cached("", "settings") {
            for (k, v) in c {
                s[k] = v;
            }
        }
        s
    }

    /// The settings from UUOB. A change made here while it was being read wins.
    pub async fn settings(&self) -> Json {
        let at = self.0.settings_changes.load(Relaxed);
        match self.call("settings", json!({})).await {
            Ok(s) if s.is_object() && at == self.0.settings_changes.load(Relaxed) => {
                self.0.store.cache("", "settings", &s);
                self.cached_settings()
            }
            _ => self.cached_settings(),
        }
    }

    /// Changes some of them, e.g. `{ "hour12": true }`; shown at once.
    pub async fn set_settings(&self, change: Json) -> Json {
        let mut next = self.cached_settings();
        if let Json::Object(c) = &change {
            for (k, v) in c {
                next[k] = v.clone();
            }
        }
        self.0.settings_changes.fetch_add(1, Relaxed);
        self.0.store.cache("", "settings", &next);
        self.emit(Event::Settings(next.clone()));
        let _ = self.call("set_settings", change).await;
        next
    }

    // ---- What is coming up ---------------------------------------------

    /// What is coming up in this app, for the user's devices to show (the
    /// app's own widget) and ring (alerts), also while the app is closed. The
    /// whole list, `[{ id, date, time?, end_date?, end?, tz?, title, body?,
    /// alert?, color? }]`, as in `uuob.js`: only what differs from UUOB's copy
    /// is written, and nothing at all when the list is the one this copy of
    /// the app gave last time.
    pub async fn upcoming(&self, list: Json) {
        let c = self.clone();
        on_runtime(async move {
            let mut given = c.0.upcoming.lock().await;
            let key = list.to_string();
            if given.as_deref() == Some(key.as_str()) {
                return;
            }
            let Ok(now) = c.call("list", json!({ "collection": "upcoming" })).await else { return };
            let now = now.as_array().cloned().unwrap_or_default();
            let mut want: Vec<(String, Map<String, Json>)> = vec![];
            for item in list.as_array().into_iter().flatten() {
                let Some(o) = item.as_object() else { continue };
                let id = match &o["id"] {
                    Json::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let a = o.iter().filter(|(k, v)| *k != "id" && !v.is_null()).map(|(k, v)| (k.clone(), v.clone())).collect();
                want.push((id, a));
            }
            for r in &now {
                let id = r["id"].as_str().unwrap_or_default();
                if !want.iter().any(|(w, _)| w == id) && c.delete("upcoming", None, id).await.is_err() {
                    return;
                }
            }
            for (id, a) in &want {
                let had: Map<String, Json> = now
                    .iter()
                    .find(|r| r["id"] == json!(id))
                    .and_then(|r| r.as_object().cloned())
                    .map(|mut m| {
                        m.remove("id");
                        m
                    })
                    .unwrap_or_default();
                if had != *a && c.put("upcoming", None, Some(id), Json::Object(a.clone())).await.is_err() {
                    return;
                }
            }
            *given = Some(key);
        })
        .await
    }
}

/// A record with the id of its container (as `uuob.all` gives them).
fn with_container(mut r: Json, container: &str) -> Json {
    if let Some(o) = r.as_object_mut() {
        o.insert("container".into(), json!(container));
    }
    r
}

/// Percent-encoding for a query value.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}
