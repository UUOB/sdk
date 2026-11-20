//! "Update to …": an app installs newer versions of itself. Each app says
//! where its updates are published and which key signs them, so every author
//! publishes their own. The feed and the signatures are the ones Tauri's
//! updater used (`crates/uuob-update`), so copies of the old web version
//! find the native one and the other way round.
//!
//! - Mac: the signed `.app.tar.gz` replaces this app; the app restarts.
//! - Android: the signed APK is saved for the system installer.
//! - Linux: the app is updated with the system's packages.

mod feed;

use serde_json::Value as Json;

/// This device's name in the feed: `darwin-aarch64`, `android-aarch64`, …
fn target() -> String {
    let os = if cfg!(target_os = "macos") { "darwin" } else { std::env::consts::OS };
    format!("{os}-{}", std::env::consts::ARCH)
}

fn get(url: &str) -> Result<Vec<u8>, String> {
    let failed = |e: ureq::Error| format!("The update couldn’t be downloaded ({e}).");
    let response = ureq::get(url).call().map_err(failed)?;
    response.into_body().with_config().limit(1 << 30).read_to_vec().map_err(failed)
}

fn newer(feed: &str, version: &str) -> Result<Option<(String, feed::File)>, String> {
    let current = semver::Version::parse(version).map_err(|e| e.to_string())?;
    feed::newer(&get(feed)?, &current, &target())
}

/// The newer version, if there is one.
pub fn check(feed: &str, version: &str) -> Result<Option<String>, String> {
    Ok(newer(feed, version)?.map(|(v, _)| v))
}

/// Downloads the newer version, checks its signature against `key`, and
/// installs it (see the module). Null when there is nothing newer.
pub fn install(feed: &str, key: &str, version: &str, params: &Json) -> Result<Json, String> {
    let Some((_, file)) = newer(feed, version)? else { return Ok(Json::Null) };
    let data = get(&file.url)?;
    feed::verify(&data, &file.signature, key)?;
    platform::install(data, params)
}

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use serde_json::json;

    pub fn install(apk: Vec<u8>, params: &Json) -> Result<Json, String> {
        let dir = params["dir"].as_str().ok_or("where to save the update is missing")?;
        let path = std::path::Path::new(dir).join("update.apk");
        std::fs::write(&path, apk).map_err(|e| e.to_string())?;
        Ok(json!({ "apk": path }))
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use serde_json::json;
    use std::path::Path;

    /// Unpacks the new app next to this one and swaps the two.
    pub fn install(archive: Vec<u8>, _: &Json) -> Result<Json, String> {
        let text = |e: std::io::Error| format!("The update couldn’t be installed ({e}).");
        let exe = std::env::current_exe().map_err(text)?;
        let app = exe.ancestors().nth(3).filter(|a| a.extension().is_some_and(|e| e == "app")).ok_or("The app isn’t running as an app bundle")?;
        let folder = app.parent().ok_or("The app isn’t running as an app bundle")?;
        let staging = folder.join(format!(".{}-update", app.file_stem().unwrap_or_default().to_string_lossy()));
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging).map_err(text)?;
        let swapped = (|| {
            tar::Archive::new(flate2::read::GzDecoder::new(&archive[..])).unpack(&staging)?;
            let new = find_app(&staging)?;
            let old = staging.join("previous.app");
            std::fs::rename(app, &old)?;
            if let Err(e) = std::fs::rename(&new, app) {
                let _ = std::fs::rename(&old, app);
                return Err(e);
            }
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&staging);
        swapped.map_err(text)?;
        Ok(json!({ "restart": app }))
    }

    fn find_app(folder: &Path) -> std::io::Result<std::path::PathBuf> {
        std::fs::read_dir(folder)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .ok_or_else(|| std::io::Error::other("the update has no app in it"))
    }
}

#[cfg(not(any(target_os = "android", target_os = "macos")))]
mod platform {
    use super::*;

    pub fn install(_: Vec<u8>, _: &Json) -> Result<Json, String> {
        Err("On Linux, update the app with your system’s packages.".into())
    }
}
