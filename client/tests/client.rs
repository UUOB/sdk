//! The native SDK against a real UUOB, as `ui/uuob.test.mjs` tests the web
//! one: connecting, reading and writing, changes made while UUOB is stopped,
//! settings and what is coming up.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use uuob_client::{App, Client, Event, State};
use uuob_core::{Config, MemoryLookup, NetMode, Node};

const PORT: u16 = 17790;

/// UUOB's local server, on a runtime of its own so that stopping it really
/// stops it (open streams included), like quitting UUOB.
fn serve(node: &Node) -> tokio::runtime::Runtime {
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap();
    let app = uuob_api::router(node.clone());
    let (tx, rx) = std::sync::mpsc::channel();
    rt.spawn(async move {
        let listener = loop {
            match tokio::net::TcpListener::bind(("127.0.0.1", PORT)).await {
                Ok(l) => break l,
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        };
        tx.send(()).unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    rx.recv().unwrap();
    rt
}

async fn until(what: &str, mut f: impl AsyncFnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while !f().await {
        assert!(Instant::now() < end, "timed out waiting for: {what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn titles(records: &[Value]) -> Vec<String> {
    let mut t: Vec<String> = records.iter().map(|r| r["title"].as_str().unwrap_or_default().to_string()).collect();
    t.sort();
    t
}

#[test]
fn reads_writes_and_sends_later_what_changed_while_uuob_was_away() {
    // SAFETY: set before any thread reads it.
    unsafe { std::env::set_var("UUOB_PORT", PORT.to_string()) };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::TempDir::new().unwrap();
    let app_dir = tempfile::TempDir::new().unwrap();
    rt.block_on(async {
        let node = Node::open(Config {
            data_dir: dir.path().into(),
            device_name: "A".into(),
            platform: "linux".into(),
            net: NetMode::Local(MemoryLookup::new()),
        })
        .await
        .unwrap();
        let shell = uuob_core::Client::Shell;
        let tasks_app = uuob_core::Client::App("tasks".into());
        let mut server = Some(serve(&node));

        let uuob = Client::new(App { id: "tasks".into(), name: "Tasks".into(), android_package: None }, app_dir.path());
        let mut events = uuob.subscribe();
        assert_eq!(uuob.check().await, State::Disconnected);
        assert!(!uuob.has_token());

        // A wrong code is refused; the right one connects.
        assert!(uuob.connect(Some("1-wrong-code-here".into())).await.is_err());
        let code = node.create_app_code().await.unwrap().code;
        uuob.connect(Some(code)).await.unwrap();
        assert!(uuob.has_token());
        assert_eq!(uuob.check().await, State::Connected);
        assert_eq!(uuob.device().as_deref(), Some("A"));

        // UUOB is running: changes go straight to it.
        uuob.put("tasks", None, Some("t1"), json!({ "title": "one", "done": false })).await.unwrap();
        uuob.put("tasks", None, Some("t2"), json!({ "title": "two", "done": false })).await.unwrap();
        uuob.put("tasks", None, Some("t3"), json!({ "title": "three", "done": false })).await.unwrap();
        assert_eq!(uuob.pending(), 0);
        let list = uuob.load("tasks", None).await.unwrap();
        assert_eq!(titles(&list), ["one", "three", "two"]);
        let (all, complete) = uuob.all("tasks").await;
        assert!(complete);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0]["container"], "tasks");
        // A change elsewhere is announced.
        node.put(&tasks_app, "tasks", "tasks", Some("t9"), json!({ "title": "nine" })).unwrap();
        until("the app hears of a change", async || matches!(events.try_recv(), Ok(Event::Changed(c)) if c == "tasks" || c == "*")).await;
        node.delete(&tasks_app, "tasks", "tasks", "t9").unwrap();

        // UUOB stops. The app keeps working on what it last saw.
        server.take().unwrap().shutdown_background();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let t1 = uuob.get("tasks", None, "t1").await.unwrap();
        let mut t1 = t1.as_object().cloned().unwrap();
        t1.insert("done".into(), json!(true));
        uuob.put("tasks", None, None, Value::Object(t1)).await.unwrap(); // one field of an existing record
        let made = uuob.put("tasks", None, None, json!({ "title": "made while away", "done": false })).await.unwrap(); // a new record
        assert_eq!(uuob.delete("tasks", None, "t2").await.unwrap(), json!(true));
        uuob.put("tasks", None, Some("t3"), json!({ "title": "three, edited", "done": false })).await.unwrap();
        assert_eq!(uuob.check().await, State::Unreachable);
        assert_eq!(uuob.pending(), 4);
        let shown = uuob.load("tasks", None).await.unwrap();
        assert_eq!(titles(&shown), ["made while away", "one", "three, edited"]);
        assert_eq!(shown.iter().find(|t| t["id"] == "t1").unwrap()["done"], true);
        let made_id = made["id"].as_str().unwrap().to_string();
        assert_eq!(made_id.len(), 16);
        assert_eq!(uuob.get("tasks", None, &made_id).await.unwrap()["title"], "made while away");
        assert_eq!(uuob.get("tasks", None, "t2").await.unwrap(), Value::Null);
        // Only what changed waits, not whole records.
        let outbox: Value = serde_json::from_slice(&std::fs::read(app_dir.path().join("uuob/outbox.json")).unwrap()).unwrap();
        assert_eq!(outbox[0], json!({ "container": "tasks", "collection": "tasks", "id": "t1", "fresh": false, "set": { "done": true }, "unset": [] }));

        // Meanwhile on another device: t1 is renamed and t3 is deleted.
        node.put(&tasks_app, "tasks", "tasks", Some("t1"), json!({ "title": "renamed elsewhere", "done": false })).unwrap();
        node.delete(&tasks_app, "tasks", "tasks", "t3").unwrap();

        // UUOB is back: what waited is sent, and both changes to t1 survive.
        server = Some(serve(&node));
        until("nothing waits any more", async || uuob.pending() == 0).await;
        let now = node.list(&tasks_app, "tasks", "tasks").unwrap();
        let by_id = |id: &str| now.iter().find(|t| t["id"] == id).cloned();
        assert_eq!(now.len(), 2);
        assert_eq!(by_id("t1").unwrap(), json!({ "id": "t1", "title": "renamed elsewhere", "done": true }));
        assert_eq!(by_id(&made_id).unwrap()["title"], "made while away");
        assert!(by_id("t2").is_none(), "the delete made while away was sent");
        assert!(by_id("t3").is_none(), "a change to a record deleted meanwhile is dropped");
        assert_eq!(titles(&uuob.load("tasks", None).await.unwrap()), ["made while away", "renamed elsewhere"]);
        assert_eq!(uuob.check().await, State::Connected);

        // Settings for all apps: 24-hour time until changed; a change reaches UUOB, and one made elsewhere reaches the app.
        let s = uuob.settings().await;
        assert_eq!((s["hour12"].clone(), s["sunday_first"].clone()), (json!(false), json!(false)));
        let s = uuob.set_settings(json!({ "hour12": true })).await;
        assert_eq!(s["hour12"], true, "shown at once");
        assert_eq!(node.call(&shell, "settings", json!({})).await.unwrap()["hour12"], true);
        node.call(&shell, "set_settings", json!({ "sunday_first": true })).await.unwrap();
        let s = uuob.settings().await;
        assert_eq!((s["hour12"].clone(), s["sunday_first"].clone()), (json!(true), json!(true)));

        // What is coming up: the whole list is handed over, and a new list replaces it.
        let upcoming = async || {
            let all = node.call(&shell, "upcoming", json!({})).await.unwrap();
            all.as_array().unwrap().iter().map(|i| format!("{}:{}:{}", i["id"].as_str().unwrap(), i["title"].as_str().unwrap(), i["body"].as_str().unwrap_or(""))).collect::<Vec<_>>().join(" ")
        };
        uuob.upcoming(json!([
            { "id": "a", "date": "2030-01-01", "time": "09:00", "title": "One", "body": "Room 2", "alert": 10 },
            { "id": "b", "date": "2030-01-02", "title": "Two" },
        ]))
        .await;
        assert_eq!(upcoming().await, "a:One:Room 2 b:Two:");
        let second = json!([
            { "id": "a", "date": "2030-01-01", "time": "09:00", "title": "One", "body": null, "alert": 10 },
            { "id": "c", "date": "2030-01-03", "time": "10:00", "tz": "Europe/Prague", "title": "Three" },
        ]);
        uuob.upcoming(second.clone()).await;
        assert_eq!(upcoming().await, "a:One: c:Three:");
        // Another copy writes its own version; handing over the same list again doesn't undo it.
        node.put(&tasks_app, "tasks", "upcoming", Some("c"), json!({ "date": "2030-01-03", "time": "10:00", "title": "Three, elsewhere" })).unwrap();
        uuob.upcoming(second).await;
        assert_eq!(upcoming().await, "a:One: c:Three, elsewhere:");

        // Containers and files.
        let id = uuob.create_container("2026-11", Some("month20261100000")).await.unwrap();
        assert_eq!(id, "tasks.month20261100000", "a key gives the same id on every device");
        let cs = uuob.containers().await.unwrap();
        assert!(cs.iter().any(|c| c["id"] == "tasks" && c["main"] == true));
        assert!(cs.iter().any(|c| c["id"] == "tasks.month20261100000" && c["name"] == "2026-11" && c["main"] == false));
        let f = uuob.put_file(Some("tasks.month20261100000"), b"hello".to_vec(), "text/plain").await.unwrap();
        let fid = f["id"].as_str().unwrap().to_string();
        assert_eq!(uuob.read_file(Some("tasks.month20261100000"), &fid).await.unwrap(), b"hello");
        assert!(uuob.file_url(Some("tasks.month20261100000"), &fid).await.contains(&format!(":{PORT}/api/files/")));
        uuob.delete_file(Some("tasks.month20261100000"), &fid).await.unwrap();
        assert!(uuob.read_file(Some("tasks.month20261100000"), &fid).await.is_err());

        if let Some(s) = server.take() {
            s.shutdown_background();
        }
    });
}
