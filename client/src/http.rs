//! Plain HTTP/1.1 to UUOB's local API on 127.0.0.1: a request with a JSON
//! (or raw) body, and the server-sent events stream.

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use std::sync::OnceLock;

fn client() -> &'static Client<HttpConnector, Full<Bytes>> {
    static C: OnceLock<Client<HttpConnector, Full<Bytes>>> = OnceLock::new();
    C.get_or_init(|| {
        let mut http = HttpConnector::new();
        http.set_connect_timeout(Some(std::time::Duration::from_secs(5)));
        Client::builder(TokioExecutor::new()).build(http)
    })
}

/// Couldn't reach UUOB at all (nothing listening, the tunnel found no device).
#[derive(Debug)]
pub struct Unreachable;

pub async fn send(
    port: u16,
    path: &str,
    token: Option<&str>,
    body: Option<(Vec<u8>, &str)>,
) -> Result<Response<Incoming>, Unreachable> {
    let mut req = Request::builder()
        .method(if body.is_some() { "POST" } else { "GET" })
        .uri(format!("http://127.0.0.1:{port}/api{path}"));
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let (bytes, ty) = body.unwrap_or_default();
    if !ty.is_empty() {
        req = req.header("Content-Type", ty);
    }
    let req = req.body(Full::new(Bytes::from(bytes))).map_err(|_| Unreachable)?;
    client().request(req).await.map_err(|_| Unreachable)
}

/// The whole body of a response.
pub async fn read(res: Response<Incoming>) -> Result<(u16, Vec<u8>), Unreachable> {
    let status = res.status().as_u16();
    let body = res.into_body().collect().await.map_err(|_| Unreachable)?.to_bytes();
    Ok((status, body.to_vec()))
}

/// The next piece of a streamed body (None at its end).
pub async fn chunk(body: &mut Incoming) -> Option<Bytes> {
    loop {
        match body.frame().await? {
            Ok(frame) => {
                if let Ok(data) = frame.into_data() {
                    return Some(data);
                }
            }
            Err(_) => return None,
        }
    }
}
