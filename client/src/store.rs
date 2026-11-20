//! What the app keeps on its own device, in its private data folder (the OS
//! keeps it encrypted at rest): its key to UUOB, the last records it saw (so
//! it shows them at once on launch and while UUOB is unreachable) and the
//! changes made while UUOB was away. `ui/uuob.js` keeps the same in
//! localStorage.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};

/// One change made while UUOB was unreachable: the fields of a record that
/// were changed (`set`, `unset`), or its removal.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Change {
    pub container: String,
    pub collection: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove: bool,
    /// The record didn't exist when the change was made.
    #[serde(default)]
    pub fresh: bool,
    #[serde(default)]
    pub set: Map<String, Json>,
    #[serde(default)]
    pub unset: Vec<String>,
}

pub struct Store {
    dir: PathBuf,
    /// Serialises reads and writes of the outbox.
    outbox: Mutex<()>,
}

fn write(path: &PathBuf, bytes: &[u8]) {
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, bytes).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        let _ = std::fs::create_dir_all(dir.join("cache"));
        Store { dir, outbox: Mutex::new(()) }
    }

    pub fn token(&self) -> Option<String> {
        std::fs::read_to_string(self.dir.join("token")).ok().map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
    }

    pub fn set_token(&self, token: Option<&str>) {
        let path = self.dir.join("token");
        match token {
            Some(t) => {
                write(&path, t.as_bytes());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
                }
            }
            None => drop(std::fs::remove_file(path)),
        }
    }

    fn cache_file(&self, container: &str, collection: &str) -> PathBuf {
        self.dir.join("cache").join(format!("{}.{}.json", hex::encode(container), hex::encode(collection)))
    }

    /// The last known records of a collection (or anything else kept under
    /// a name: `("", "settings")`, `("", "containers")`).
    pub fn cached(&self, container: &str, collection: &str) -> Option<Json> {
        let bytes = std::fs::read(self.cache_file(container, collection)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub fn cache(&self, container: &str, collection: &str, value: &Json) {
        write(&self.cache_file(container, collection), value.to_string().as_bytes());
    }

    pub fn clear_cache(&self) {
        let _ = std::fs::remove_dir_all(self.dir.join("cache"));
        let _ = std::fs::create_dir_all(self.dir.join("cache"));
    }

    pub fn outbox(&self) -> Vec<Change> {
        let _g = self.outbox.lock().unwrap();
        self.read_outbox()
    }

    fn read_outbox(&self) -> Vec<Change> {
        std::fs::read(self.dir.join("outbox.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    fn save_outbox(&self, list: &[Change]) {
        let path = self.dir.join("outbox.json");
        if list.is_empty() {
            let _ = std::fs::remove_file(path);
        } else {
            write(&path, &serde_json::to_vec(list).unwrap_or_default());
        }
    }

    pub fn push(&self, change: Change) {
        let _g = self.outbox.lock().unwrap();
        let mut list = self.read_outbox();
        list.push(change);
        self.save_outbox(&list);
    }

    /// Takes the oldest change off the list (after it was sent).
    pub fn shift(&self) {
        let _g = self.outbox.lock().unwrap();
        let mut list = self.read_outbox();
        if !list.is_empty() {
            list.remove(0);
        }
        self.save_outbox(&list);
    }

    pub fn clear_outbox(&self) {
        let _g = self.outbox.lock().unwrap();
        self.save_outbox(&[]);
    }
}

/// Records as they look with the waiting changes applied.
pub fn with_pending(outbox: &[Change], container: &str, collection: &str, records: Vec<Json>) -> Vec<Json> {
    let mut records = records;
    for e in outbox {
        if e.container != container || e.collection != collection {
            continue;
        }
        let i = records.iter().position(|r| r["id"] == Json::String(e.id.clone()));
        if e.remove {
            if let Some(i) = i {
                records.remove(i);
            }
        } else if i.is_some() || e.fresh {
            let mut r = i.and_then(|i| records[i].as_object().cloned()).unwrap_or_default();
            for (k, v) in &e.set {
                r.insert(k.clone(), v.clone());
            }
            r.insert("id".into(), Json::String(e.id.clone()));
            for k in &e.unset {
                r.remove(k);
            }
            match i {
                Some(i) => records[i] = Json::Object(r),
                None => records.push(Json::Object(r)),
            }
        }
    }
    records
}
