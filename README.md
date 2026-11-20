# UUOB SDK

What a native app needs to keep its data in UUOB. Calendar, Tasks and Photos use it, and any other app can too.

- **`client/`** is the SDK itself, in Rust: connect to UUOB, read and write the app's data, follow changes, post alerts, update the app. Kotlin and Swift get it through UniFFI.
- **`kit/`** holds the screens every app shares, one set per platform: the "Connect to UUOB" screen, Settings, the time picker, the theme and the updater.
  - `kit/android`: Jetpack Compose
  - `kit/mac`: SwiftUI
  - `kit/linux`: GTK 4, Libadwaita and Relm4. It is a separate Cargo workspace because it needs GTK.

UUOB itself doesn't use this repository. The SDK uses UUOB's core as a library, pinned to one commit in `client/Cargo.toml`, for two things: the tunnel to a UUOB on another device, and the types it shares with UUOB. Nothing else is copied from UUOB.

## In an app

Add it as a submodule:

```bash
git submodule add https://github.com/UUOB/sdk sdk
```

Then point the app's builds at it:

- Cargo: `uuob-client = { path = "../sdk/client" }`, plus `uuob-kit-gtk = { path = "../sdk/kit/linux" }` on Linux
- Gradle: `kotlin.srcDir("../../sdk/kit/android/src")`
- The Mac: compile `sdk/kit/mac/*.swift` with the app

## Tests

```bash
cargo test
```

The tests run a real UUOB in the test process, so they also check that the SDK still matches the pinned UUOB.

## License

AGPL-3.0-or-later, like UUOB.
