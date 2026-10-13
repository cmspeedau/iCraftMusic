//! MusicCraft — a native desktop music player written entirely in Rust.
//!
//! The crate is split into a library and a thin binary (`src/main.rs`) so the
//! application modules can be exercised by `cargo test` without opening a
//! desktop window.
//!
//! Module layout follows `AGENT.md` §4; `ROADMAP.md` says what each stage
//! delivers and which parts are still missing. Application code is Rust only —
//! the modules under `audio`, `library`, `database`, `playlists` and `search`
//! hold the behaviour, and `ui` draws it.

pub mod app;
pub mod audio;
pub mod config;
pub mod database;
pub mod diagnostics;
pub mod error;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod library;
pub mod logging;
pub mod playlists;
pub mod search;
pub mod ui;

/// Human-readable application name, used for window titles and config paths.
pub const APP_NAME: &str = "MusicCraft";

/// Application version, taken from `Cargo.toml` at compile time.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
