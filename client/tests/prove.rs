//! On Android, another app could take UUOB's port while UUOB is not running.
//! The SDK sends nothing to the port until the server there answers a nonce
//! as UUOB answers it through the system (the prover). Its own process: the
//! port is set through UUOB_PORT.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::Request;
use axum::routing::get;
use serde_json::json;
use uuob_client::{App, Client, State};
use uuob_core::{Config, MemoryLookup, NetMode, Node};

const PORT: u16 = 17795;

#[test]
fn nothing_goes_to_a_port_that_is_not_uuob() {
    // SAFETY: set before any thread reads it.
    unsafe { std::env::set_var("UUOB_PORT", PORT.to_string()) };
    let app_dir = tempfile::TempDir::new().unwrap();
    let dir = tempfile::TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Another app holds the port and answers like UUOB, without the proof.
    let seen: Arc<Mutex<Vec<String>>> = Default::default();
    let fake_rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap();
    let log = seen.clone();
    let fake = Router::new()
        .route("/api", get(|| async { axum::Json(json!({ "uuob": "0.4.0", "device": "Phone", "remote": false })) }))
        .fallback(move |r: Request| {
            let log = log.clone();
            async move {
                log.lock().unwrap().push(format!("{} {}", r.method(), r.uri().path()));
                axum::Json(json!({ "token": "stolen" }))
            }
        });
    fake_rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", PORT)).await.unwrap();
        tokio::spawn(async move { axum::serve(listener, fake).await });
    });

    let client = Client::new(App { id: "tasks".into(), name: "Tasks".into(), android_package: None }, app_dir.path());
    // What UUOB on this phone would answer through the system.
    client.set_prover(Arc::new(|nonce: String| uuob_api::proof(&nonce)));
    rt.block_on(async {
        assert_eq!(client.check().await, State::Unreachable);
        let err = client.connect(None).await.unwrap_err();
        assert_eq!(err.code(), "unreachable");
        assert!(client.file_url(None, "x").await.is_empty(), "no address with the key");
    });
    assert!(seen.lock().unwrap().is_empty(), "nothing was sent to the impostor: {:?}", seen.lock().unwrap());
    drop(fake_rt);

    // The real UUOB on the port: it proves itself, and connecting works.
    rt.block_on(async {
        let node = Node::open(Config {
            data_dir: dir.path().into(),
            device_name: "Phone".into(),
            platform: "android".into(),
            net: NetMode::Local(MemoryLookup::new()),
        })
        .await
        .unwrap();
        let code = node.create_app_code().await.unwrap().code;
        tokio::spawn(uuob_api::serve(node, ([127, 0, 0, 1], PORT).into()));
        let mut state = State::Unreachable;
        for _ in 0..100 {
            state = client.check().await;
            if state != State::Unreachable {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(state, State::Disconnected);
        client.connect(Some(code)).await.unwrap();
        assert!(client.has_token());
        assert!(client.file_url(None, "x").await.contains("key="));
    });
}
